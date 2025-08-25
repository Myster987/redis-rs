use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid bytes")]
    InvalidBytes,
    #[error("request is incomplete")]
    Incomplete,
    #[error("unknown command type")]
    UnknownCommand,
    #[error("connection was closed")]
    ConnectionClosed,

    #[error("bad request: {msg}")]
    BadRequest { msg: String },
    #[error("database responded with error: {msg}")]
    DatabaseError { msg: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
