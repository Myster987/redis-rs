use std::io::{Cursor, Read};

use crate::{
    error::Error,
    server::protocol,
    utils::bytes::{get_bool, get_i64, get_u8, get_u32},
};

#[derive(Debug, Clone)]
pub enum Value {
    /// represeted as !
    Boolean(bool),
    /// represeted as #
    Number(i64),
    /// represeted as $ and len included
    String(String),
    /// represeted as [ and len included
    Array(Vec<Value>),
    /// represeted as { and len included
    Bytes(Vec<u8>),
}

impl Value {
    pub fn to_bytes(self) -> Vec<u8> {
        match self {
            Self::Boolean(b) => vec![b'!', b as u8],
            Self::Number(n) => {
                let mut encoded = vec![b'#'];
                encoded.extend_from_slice(&n.to_le_bytes());
                encoded
            }
            Self::String(s) => {
                let mut encoded = vec![b'$'];
                let len = s.len() as u32;
                encoded.extend_from_slice(&len.to_le_bytes());
                encoded.extend_from_slice(s.as_bytes());
                encoded
            }
            Self::Array(arr) => {
                let mut encoded = vec![b'['];
                let len = arr.len() as u32;
                encoded.extend_from_slice(&len.to_le_bytes());
                for el in arr {
                    encoded.extend_from_slice(&el.to_bytes());
                }
                encoded
            }
            Self::Bytes(bytes) => {
                let mut encoded = vec![b'{'];
                let len = bytes.len() as u32;
                encoded.extend_from_slice(&len.to_le_bytes());
                encoded.extend_from_slice(&bytes);
                encoded
            }
        }
    }
}

impl protocol::TcpRead for Value {
    fn validate(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        let valute_type = get_u8(src)?;
        if matches!(valute_type, b'!' | b'#' | b'$' | b'[' | b'{') {
            get_line(src)?;
            Ok(())
        } else {
            Err(Error::UnknownCommand)
        }
    }

    // Is it pretty? I don't think so, but it works like a charm :D
    fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let data_type = get_u8(src)?;

        match data_type {
            b'!' => Ok(Value::Boolean(get_bool(src)?)),
            b'#' => Ok(Value::Number(get_i64(src)?)),
            b'$' => {
                let len = get_u32(src)?;
                let mut string_buf = vec![0; len as usize];
                src.read_exact(&mut string_buf)?;
                Ok(Value::String(String::from_utf8(string_buf).map_err(
                    |_| Error::BadRequest {
                        msg: "Invalid key utf-8 encoding".into(),
                    },
                )?))
            }
            b'[' => {
                let len = get_u32(src)?;
                let mut arr = Vec::with_capacity(len as usize);

                for _ in 0..len {
                    arr.push(Value::parse(&mut *src)?);
                }

                Ok(Value::Array(arr))
            }
            b'{' => {
                let len = get_u32(src)?;
                let mut bytes = vec![0; len as usize];
                src.read_exact(&mut bytes)?;
                Ok(Value::Bytes(bytes))
            }
            _ => Err(Error::BadRequest {
                msg: "Invalid data type".into(),
            }),
        }
    }
}

#[derive(Debug)]
pub enum CommandType {
    Get,
    Set { value: Value },
    Delete,
}

#[derive(Debug)]
pub struct Command {
    /// Common filed for all commands
    pub key: String,
    pub r#type: CommandType,
}

impl Command {
    pub fn get(key: &str) -> Self {
        Self {
            key: key.to_string(),
            r#type: CommandType::Get,
        }
    }

    pub fn set(key: &str, value: Value) -> Self {
        Self {
            key: key.to_string(),
            r#type: CommandType::Set { value },
        }
    }
    pub fn delete(key: &str) -> Self {
        Self {
            key: key.to_string(),
            r#type: CommandType::Delete,
        }
    }

    fn byte_type(&self) -> u8 {
        match &self.r#type {
            CommandType::Get => b'g',
            CommandType::Set { value: _ } => b's',
            CommandType::Delete => b'd',
        }
    }

    fn validate_command_type(command_type: u8) -> Result<(), Error> {
        if !matches!(command_type, b'g' | b's' | b'd') {
            return Err(Error::UnknownCommand);
        }
        Ok(())
    }
}

impl protocol::TcpRead for Command {
    fn validate(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        let command_type = get_u8(src)?;
        if matches!(command_type, b'g' | b's' | b'd') {
            get_line(src)?;
            Ok(())
        } else {
            Err(Error::UnknownCommand)
        }
    }

    fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, Error> {
        let command_type = get_u8(src)?;

        Self::validate_command_type(command_type)?;

        let key_size = get_u32(src)?;
        let mut key_buf = vec![0; key_size as usize];

        src.read_exact(&mut key_buf)?;

        let key = String::from_utf8(key_buf).map_err(|_| Error::BadRequest {
            msg: "Invalid key utf-8 encoding".into(),
        })?;

        let r#type = match command_type {
            b'g' => CommandType::Get,
            b'd' => CommandType::Delete,
            b's' => {
                let value = Value::parse(src)?;
                CommandType::Set { value }
            }
            _ => unreachable!(),
        };

        Ok(Command { key, r#type })
    }
}

impl protocol::TcpWrite for Command {
    fn to_bytes(self) -> Vec<u8> {
        let mut encoded = vec![self.byte_type()];

        let key_size = self.key.len() as u32;
        encoded.extend_from_slice(&key_size.to_le_bytes());
        encoded.extend_from_slice(self.key.as_bytes());

        if let CommandType::Set { value } = self.r#type {
            encoded.extend_from_slice(&value.to_bytes());
        }

        encoded.extend_from_slice(b"\r\n");

        encoded
    }
}

pub fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            // advance src to go over request separator
            src.set_position((i + 2) as u64);
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(Error::Incomplete)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value() -> anyhow::Result<()> {
        let val_1 = Value::Number(1);
        let val_2 = Value::String("maciek".into());
        let val_3 = Value::Number(3);
        let value = Value::Array(vec![val_1, val_2, val_3]);

        let bytes = value.to_bytes();

        println!("{:?}", bytes);

        // println!("{:?}", Value::parse_value(bytes.as_slice()));

        Ok(())
    }

    // #[test]
    // fn test_command() -> anyhow::Result<()> {
    //     let val_1 = Value::Number(1);
    //     let val_2 = Value::String("maciek".into());
    //     let val_3 = Value::Number(3);
    //     let value = Value::Array(vec![val_1, val_2, val_3]);

    //     let command = Command {
    //         key: "maciek".into(),
    //         r#type: CommandType::Set { value },
    //     };

    //     let bytes = command.to_bytes();

    //     println!("{:?}", bytes);

    //     println!("{:?}", Command::parse_command(bytes.as_slice())?);

    //     Ok(())
    // }
}
