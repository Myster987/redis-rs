use tokio::net::TcpStream;

use crate::{
    error::Error,
    server::protocol::{Connection, Response},
    utils::command::{Command, Value},
};

pub struct Client {
    connection: Connection,
}

fn response_to_value(response: Result<Response, Error>) -> Result<Option<Value>, Error> {
    match response {
        Ok(Response::Payload(val)) => Ok(Some(val)),
        Ok(Response::Null) => Ok(None),
        Ok(Response::Error(msg)) => Err(Error::DatabaseError { msg }),
        Err(e) => Err(e),
    }
}

impl Client {
    pub async fn connect(to: &str) -> Result<Self, Error> {
        let stream = TcpStream::connect(to).await?;

        let connection = Connection::new(stream);

        Ok(Self { connection })
    }

    pub async fn execute(&mut self, command: Command) -> Result<(), Error> {
        self.connection.write(command).await
    }

    fn flatten_response_to_option(
        result: Result<Option<Response>, Error>,
    ) -> Result<Option<Value>, Error> {
        response_to_value(result.and_then(|opt| opt.ok_or(Error::ConnectionClosed)))
    }

    pub async fn try_get(&mut self, key: &str) -> Result<Option<Value>, Error> {
        let command = Command::get(key);
        self.execute(command).await?;

        Self::flatten_response_to_option(self.connection.read().await)
    }

    pub async fn get(&mut self, key: &str) -> Option<Value> {
        self.try_get(key).await.unwrap()
    }

    pub async fn try_set(&mut self, key: &str, value: Value) -> Result<Option<Value>, Error> {
        let command = Command::set(key, value);
        self.execute(command).await?;

        Self::flatten_response_to_option(self.connection.read().await)
    }
    pub async fn set(&mut self, key: &str, value: Value) -> Option<Value> {
        self.try_set(key, value).await.unwrap()
    }

    pub async fn try_delete(&mut self, key: &str) -> Result<Option<Value>, Error> {
        let command = Command::delete(key);
        self.execute(command).await?;

        Self::flatten_response_to_option(self.connection.read().await)
    }

    pub async fn delete(&mut self, key: &str) -> Option<Value> {
        self.try_delete(key).await.unwrap()
    }
}
