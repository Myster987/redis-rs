use crate::{error::DatabaseResult, utils::config::Config};

#[derive(Debug)]
pub struct Database {
    config: Config,
}

impl Database {
    pub fn new() -> DatabaseResult<Self> {
        let config = Config::load()?;

        Ok(Self { config })
    }
}
