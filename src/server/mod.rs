use tokio::net::TcpListener;

use crate::{error::DatabaseResult, utils::config::Config};

#[derive(Debug)]
pub struct Database {
    config: Config,
    listener: TcpListener
}

impl Database {
    pub async fn new() -> DatabaseResult<Self> {
        let config = Config::load()?;
        let listener = TcpListener::bind(format!("{}:{}", config.HOST, config.PORT)).await?;

        Ok(Self { config, listener })
    }   

    pub async fn listen(&self) -> DatabaseResult<()> {
        loop {
            let (socket, _) = self.listener.accept().await.unwrap();
        }

        Ok(())
    }
}
