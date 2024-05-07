use std::{net::SocketAddr, str::FromStr};

use anyhow::anyhow;
use tokio::{
    io::{copy_bidirectional, AsyncReadExt, AsyncWriteExt, BufStream},
    net::{TcpListener, TcpStream},
};
use tracing::{Instrument, Span};

const IPV4: u8 = 0x01;
const HOST: u8 = 0x03;
const IPV6: u8 = 0x04;

const IPV4_LEN: usize = 4;
const IPV6_LEN: usize = 16;

const SOCKS5_VER: u8 = 0x05;
const CMD_CONNECT: u8 = 0x01;

pub async fn handle(listener: TcpListener) {
    while let Ok((inbound, client_addr)) = listener.accept().await {
        if let Err(e) = handle_connection(inbound, client_addr).await {
            tracing::warn!("Can't handle this connection: {}", e);
        }
    }
}

#[tracing::instrument(name = "Conn", skip_all, fields(from = %client_addr))]
async fn handle_connection(
    inbound: TcpStream,
    client_addr: std::net::SocketAddr,
) -> anyhow::Result<()> {
    tracing::info!("New connection from: {}", client_addr);

    let mut inbound = BufStream::new(inbound);
    handshake(&mut inbound).await?;

    let remote_addr = read_address(&mut inbound).await?;

    inbound
        .write_all(&[
            0x05, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x68, 0x68,
        ])
        .await?;
    inbound.flush().await?;

    tracing::debug!("Connecting to {}", remote_addr);
    let mut outbound = TcpStream::connect(remote_addr).await?;

    tokio::spawn(
        async move {
            if let Err(e) =
                copy_bidirectional(&mut inbound, &mut outbound).await
            {
                tracing::error!("Failed to proxy; error={}", e);
            }
            tracing::info!("Closing connection to {}", remote_addr);
        }
        .instrument(Span::current()),
    );
    Ok(())
}

#[allow(clippy::indexing_slicing)]
async fn handshake(inbound: &mut BufStream<TcpStream>) -> anyhow::Result<()> {
    let mut req_buffer: Vec<u8> = vec![0; 255];

    let version = inbound.read_u8().await?;
    if version != SOCKS5_VER {
        return Err(anyhow!("Only support socks version 5"));
    }

    let nauth = inbound.read_u8().await?;
    let message_len: usize = nauth.into();

    inbound.read_exact(&mut req_buffer[..message_len]).await?;
    inbound.write_all(&[SOCKS5_VER, 0x00]).await?;
    inbound.flush().await?;

    Ok(())
}

#[allow(clippy::indexing_slicing)]
async fn read_address(
    inbound: &mut BufStream<TcpStream>,
) -> anyhow::Result<SocketAddr> {
    let mut req_buffer: Vec<u8> = vec![0; 255];

    let version = inbound.read_u8().await?;
    if version != SOCKS5_VER {
        return Err(anyhow!("Only support socks version 5"));
    }

    let command = inbound.read_u8().await?;
    if command != CMD_CONNECT {
        return Err(anyhow!("Only support command 'connect'"));
    }

    let _reserved = inbound.read_u8().await?;

    let address_type = inbound.read_u8().await?;
    let address_len: usize = match address_type {
        IPV4 => IPV4_LEN,
        HOST => inbound.read_u8().await?.into(),
        IPV6 => IPV6_LEN,
        _ => return Err(anyhow!("Unsupported address type")),
    };

    inbound.read_exact(&mut req_buffer[..address_len]).await?;
    let port = inbound.read_u16().await?;

    match address_type {
        IPV4 => {
            let addr: [u8; IPV4_LEN] =
                (&req_buffer[..address_len]).try_into()?;
            Ok(SocketAddr::from((addr, port)))
        },
        HOST => {
            let mut addr =
                String::from_utf8(req_buffer[..address_len].to_vec())?;
            addr.push(':');
            addr.push_str(format!("{port}").as_str());
            Ok(SocketAddr::from_str(&addr)?)
        },
        IPV6 => {
            let addr: [u8; IPV6_LEN] =
                (&req_buffer[..address_len]).try_into()?;
            Ok(SocketAddr::from((addr, port)))
        },
        _ => Err(anyhow!("Unsupported address type")),
    }
}
