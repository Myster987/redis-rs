use server::Database;

pub mod error;
pub mod server;
pub mod utils;
pub mod shared;
pub mod tcp;
pub mod frame;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let db = Database::new().await?;

    log::info!("{:?}", db);

    Ok(())
}
