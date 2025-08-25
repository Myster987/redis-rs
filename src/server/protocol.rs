//! General protocol encoding:
//!
//! ```text
//! +---------------------+---------+---------+------------+-----------+
//! | type - 1 byte       | GET (g) | SET (s) | DELETE (d) | ERROR (e) |
//! +---------------------+---------+---------+------------+-----------+
//! | key len - 4 bytes   |   yes   |   yes   |    yes     |    yes    |
//! +---------------------+---------+---------+------------+-----------+
//! | key - n bytes       |   yes   |   yes   |    yes     |    yes    |
//! +---------------------+---------+---------+------------+-----------+
//! | data type - 1 byte  |   no    |   yes   |    no      |    no     |
//! +---------------------+---------+---------+------------+-----------+
//! | value len - 4 bytes |   no    | depends |    no      |    no     |
//! +---------------------+---------+---------+------------+-----------+
//! | value - n butes     |   no    |   yes   |    no      |    no     |
//! +---------------------+---------+---------+------------+-----------+
//! | separator - 2 bytes |   yes   |   yes   |    yes     |    yes    |
//! +---------------------+---------+---------+------------+-----------+
//! ```

use std::{
    io::{Cursor, Read},
    net::SocketAddr,
    sync::Arc,
};

use bytes::{Buf, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use crate::{
    error::Error,
    server::storage::Database,
    utils::{
        bytes::{get_u8, get_u32},
        command::{Command, CommandType, Value, get_line},
    },
};

pub fn handle_connection(addr: SocketAddr, stream: TcpStream, db_clone: Arc<Database<String>>) {
    let mut conn = Connection::new(stream);
    tokio::spawn(async move {
        loop {
            match conn.read::<Command>().await {
                Ok(Some(command)) => {
                    log::debug!("{:?}", command);
                    match command.r#type {
                        CommandType::Get => {
                            let response = Response::new(db_clone.get(&command.key));
                            let _ = conn.write(response).await;
                        }
                        CommandType::Set { value } => {
                            let response = Response::new(db_clone.set(command.key, value));
                            let _ = conn.write(response).await;
                        }
                        CommandType::Delete => {
                            let response = Response::new(db_clone.delete(&command.key));
                            // test error to check if it would parse response correctly
                            // let response = Response::error("try again :D");
                            let _ = conn.write(response).await;
                        }
                    }
                }
                Ok(None) => {
                    log::info!("Connection from {} closed", addr);
                    break;
                }
                Err(e) => log::error!("Error: {}", e),
            }
        }
    });
}

pub trait TcpRead {
    /// This function should validate if incoming request is correct and advance cursor position to go over request len.
    fn validate(src: &mut Cursor<&[u8]>) -> Result<(), Error>;

    fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, Error>
    where
        Self: Sized;
}

pub trait TcpWrite {
    /// Anything that implements `to_bytes` can be send over tcp. The problem is if it can be later parsed safely.
    fn to_bytes(self) -> Vec<u8>;
}

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    fn try_read_request<T: TcpRead>(&mut self) -> Result<Option<T>, Error> {
        if self.buffer.is_empty() {
            return Ok(None);
        }
        let mut cursor = Cursor::new(&self.buffer[..]);

        match T::validate(&mut cursor) {
            Ok(_) => {
                let req_len = cursor.position() as usize;
                // reset position because command validation advanced it
                cursor.set_position(0);

                let request = T::parse(&mut cursor)?;

                self.buffer.advance(req_len);

                Ok(Some(request))
            }
            Err(Error::InvalidBytes) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn read<T: TcpRead>(&mut self) -> Result<Option<T>, Error> {
        loop {
            if let Ok(Some(command)) = self.try_read_request() {
                return Ok(Some(command));
            }

            if self.stream.read_buf(&mut self.buffer).await? == 0 {
                // Connection closed
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(Error::BadRequest {
                        msg: "Request was interrupted".into(),
                    });
                }
            }
        }
    }

    pub async fn write<T: TcpWrite>(&mut self, data: T) -> Result<(), Error> {
        let bytes_to_send = data.to_bytes();

        self.stream.write_all(&bytes_to_send).await?;
        self.stream.flush().await?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum Response {
    /// State if response contains some data.
    Payload(Value),
    /// State if response is error
    Error(String),
    /// State if response is empty or searched key was not found.
    Null,
}

impl Response {
    pub fn new(payload: Option<Value>) -> Self {
        if let Some(val) = payload {
            Self::Payload(val)
        } else {
            Self::Null
        }
    }

    pub fn error(msg: &str) -> Self {
        Self::Error(msg.to_string())
    }
}

impl TcpRead for Response {
    fn validate(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        let response_type = get_u8(src)?;

        match response_type {
            b'!' | b'#' | b'$' | b'[' | b'{' => {
                src.set_position(src.position() - 1);
                Value::validate(src)
            }
            b'-' => {
                get_line(src)?;
                Ok(())
            }
            b'e' => {
                get_line(src)?;
                Ok(())
            }
            _ => Err(Error::UnknownCommand),
        }
    }
    fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let response_type = get_u8(src)?;

        match response_type {
            b'!' | b'#' | b'$' | b'[' | b'{' => {
                src.set_position(src.position() - 1);
                Ok(Response::Payload(Value::parse(src)?))
            }
            b'-' => Ok(Response::Null),
            b'e' => {
                let msg_len = get_u32(src)?;
                let mut msg_buf = vec![0; msg_len as usize];

                std::io::Read::read_exact(src, &mut msg_buf)?;

                let msg = String::from_utf8(msg_buf).map_err(|_| Error::InvalidBytes)?;

                Ok(Response::Error(msg))
            }
            _ => Err(Error::UnknownCommand),
        }
    }
}

impl TcpWrite for Response {
    fn to_bytes(self) -> Vec<u8> {
        match self {
            Self::Payload(data) => {
                let mut encoded = data.to_bytes();
                encoded.extend_from_slice(b"\r\n");
                encoded
            }
            Self::Error(msg) => {
                let mut encoded = vec![b'e'];
                let len = msg.len() as u32;
                encoded.extend_from_slice(&len.to_le_bytes());
                encoded.extend_from_slice(msg.as_bytes());
                encoded.extend_from_slice(b"\r\n");
                encoded
            }
            Self::Null => vec![b'-', b'\r', b'\n'],
        }
    }
}
