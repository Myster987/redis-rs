use std::env;

use crate::error::{DatabaseError, DatabaseResult};

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct Config {
    pub HOST: String,
    pub PORT: u16,
}

impl Config {
    pub fn load() -> DatabaseResult<Self> {
        dotenvy::dotenv().ok();

        let host = env::var("HOST").unwrap_or("127.0.0.1".to_string());

        let port = env::var("PORT")
            .unwrap_or("8989".to_string())
            .parse::<u16>()
            .map_err(DatabaseError::InvalidPortFormat)?;

        if port < 1024 || port > 49151 {
            Err(DatabaseError::InvalidPortValue(port))?
        }


        Ok(Self {
            HOST: host,
            PORT: port,
        })
    }
}
