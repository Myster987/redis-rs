use std::{env, process::exit};

mod client;
mod error;
mod server;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::init()?;
    dotenvy::dotenv()?;

    let host_key = "HOST";
    let port_key = "PORT";

    let host = match env::var(host_key) {
        Ok(h) => h,
        Err(_) => {
            log::error!("Expected \"{host_key}\" env.");
            exit(1);
        }
    };
    let port = match env::var(port_key) {
        Ok(p) => p,
        Err(_) => {
            log::error!("Expected \"{port_key}\" env.");
            exit(1);
        }
    };

    server::start(&format!("{host}:{port}")).await?;

    Ok(())
}
