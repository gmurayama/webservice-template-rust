mod server;

use std::net::TcpListener;

use server::setup_server;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7000")?;
    let server = setup_server(listener).await?;
    server.await?;

    Ok(())
}
