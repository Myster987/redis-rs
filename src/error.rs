use thiserror::Error;

pub type DatabaseResult<T> = std::result::Result<T, DatabaseError>;

#[derive(Debug, Error)]
pub enum DatabaseError {
    // config
    #[error("invalid port format")]
    InvalidPortFormat(#[from] std::num::ParseIntError),

    #[error("invalid port value, please use port in range 1024 to 49151 insted of {0}")]
    InvalidPortValue(u16),

    // unknown
    #[error("unknown error")]
    Unknown,
}
