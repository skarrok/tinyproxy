use std::net::SocketAddr;

use tokio::io::copy_bidirectional;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tracing::Instrument;
use tracing::Span;

pub async fn handle(listener: TcpListener, remote_address: SocketAddr) {
    while let Ok((inbound, client_addr)) = listener.accept().await {
        if let Err(e) =
            handle_connection(inbound, client_addr, remote_address).await
        {
            tracing::warn!("Can't handle this connection: {}", e);
        }
    }
}

#[tracing::instrument(name = "Conn", skip_all, fields(from = %client_addr))]
async fn handle_connection(
    mut inbound: TcpStream,
    client_addr: std::net::SocketAddr,
    remote_addr: SocketAddr,
) -> Result<(), anyhow::Error> {
    let mut outbound = TcpStream::connect(remote_addr).await?;
    tracing::info!("Opening connection to {}", remote_addr);
    tokio::spawn(
        async move {
            if let Err(e) =
                copy_bidirectional(&mut inbound, &mut outbound).await
            {
                tracing::error!("Failed to transfer; error={}", e);
            }
            tracing::info!("Closing connection to {}", client_addr);
        }
        .instrument(Span::current()),
    );
    Ok(())
}
