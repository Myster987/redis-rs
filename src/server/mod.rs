use std::sync::Arc;

use tokio::net::TcpListener;

pub mod protocol;
pub mod storage;

pub async fn start(addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await.unwrap();
    let db = Arc::new(storage::Database::new());

    log::info!("Listening on: {}", listener.local_addr().unwrap());
    loop {
        let (conn, conn_addr) = listener.accept().await?;
        log::info!("Accepted connection from: {}", conn_addr);
        protocol::handle_connection(conn_addr, conn, db.clone());
    }
}
