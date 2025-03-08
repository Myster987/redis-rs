use server::Database;

pub mod error;
pub mod server;
pub mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let db = Database::new()?;

    log::info!("{:?}", db);

    Ok(())
}
