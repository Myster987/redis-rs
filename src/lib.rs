mod client;
mod error;
mod server;
mod utils;

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use tokio::time::sleep;

    use crate::{client::Client, utils::command::Value};

    #[tokio::test]
    async fn test_client() -> anyhow::Result<()> {
        simple_logger::init()?;
        let mut client = Client::connect("127.0.0.1:5000").await?;

        // let val_1 = Value::Number(1);
        // let val_2 = Value::String("maciek".into());
        // let val_3 = Value::Number(3);
        // let value = Value::Array(vec![val_1, val_2, val_3]);

        // let command = Command::get("maciek");

        // client.execute(command).await;

        let res = client
            .try_set("maciek", Value::String("kowalski".into()))
            .await?;
        log::debug!("res: {:?}", res);

        let res = client.try_delete("maciek").await;
        log::debug!("res: {:?}", res);

        sleep(Duration::from_millis(3_000)).await;

        let res = client.try_get("maciek").await?;
        log::debug!("res: {:?}", res);

        Ok(())
    }
}
