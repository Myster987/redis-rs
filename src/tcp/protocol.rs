//! Simple protocol inspired by RESP (Redis Serialization Protocol)
//!
//! ## Client
//!
//! *raw client request structure:*
//! ```text
//! +--------------+---------+--------------+---------------------------+-----------------+
//! | message type | version | content type | content length (optional) | content         |
//! |--------------|---------|--------------|---------------------------+-----------------+
//! | 1 byte       | 2 byte  | 1 bytes      | 4 bytes                   | variable length |
//! +--------------+---------+--------------+---------------------------+-----------------+
//! ```
//!
//! message type -> symbol:
//! - OPEN CONNECTION -> (
//! - CLOSE CONNECTION -> )
//! - GET -> g
//! - SET -> s
//! - DELETE -> d
//!

use std::io::{Cursor, Read};

use bytes::{Buf, Bytes};

use crate::error::{DatabaseError, DatabaseResult};

pub enum MessageType {
    Get,
    Set,
    Delete,
}

impl MessageType {
    fn from_byte(value: u8) -> DatabaseResult<Self> {
        match value {
            b'g' => Ok(MessageType::Get),
            b's' => Ok(MessageType::Set),
            b'd' => Ok(MessageType::Delete),
            _ => Err(DatabaseError::InvalidRequest),
        }
    }
}

pub enum ContentType {
    Number,
    String,
    Boolean,
    Bytes,
    Array,
}

impl ContentType {
    fn from_byte(value: u8) -> DatabaseResult<Self> {
        match value {
            0 => Ok(ContentType::Number),
            1 => Ok(ContentType::String),
            3 => Ok(ContentType::Boolean),
            4 => Ok(ContentType::Bytes),
            5 => Ok(ContentType::Array),
            _ => Err(DatabaseError::InvalidRequest),
        }
    }
}

pub enum Content {
    Number(i64),
    String(String),
    Boolean(bool),
    Bytes(Bytes),
    Array(Vec<Content>),
}

pub struct Request {
    message_type: MessageType,
    version: u16,
    content: Content,
}

impl Request {
    pub fn from_cursor(cursor: &mut Cursor<&[u8]>) -> DatabaseResult<Self> {
        let message_type = get_message_type(cursor)?;
        let version = get_version(cursor)?;
        let 

        match message_type {
            M
        }
    }
}

fn get_message_type(cursor: &mut Cursor<&[u8]>) -> DatabaseResult<MessageType> {
    Ok(MessageType::from_byte(get_u8(cursor)?)?)
}

fn get_version(cursor: &mut Cursor<&[u8]>) -> DatabaseResult<u16> {
    if !cursor.has_remaining() {
        return Err(DatabaseError::InvalidRequest);
    }
    Ok(cursor.get_u16())
}

fn get_content_type(cursor: &mut Cursor<&[u8]>) -> DatabaseResult<ContentType> {
    Ok(ContentType::from_byte(get_u8(cursor)?)?)
}

fn get_u8(cursor: &mut Cursor<&[u8]>) -> DatabaseResult<u8> {
    if !cursor.has_remaining() {
        return Err(DatabaseError::InvalidRequest);
    }

    Ok(cursor.get_u8())
}
