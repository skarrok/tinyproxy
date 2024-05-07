use anyhow::anyhow;
use http::Request;
use tokio::{
    io::{copy_bidirectional, AsyncReadExt, AsyncWriteExt, BufStream},
    net::{TcpListener, TcpStream},
};
use tracing::{Instrument, Span};

const CONNECTION_ESTABLISHED: &[u8; 39] =
    b"HTTP/1.1 200 Connection established\r\n\r\n";

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
    let mut req_buffer: Vec<u8> = Vec::with_capacity(4096);

    let request = parse(&mut inbound, &mut req_buffer).await?;

    tracing::debug!("Parsed {:?}", request);

    let Some(host) = request.headers().get("host") else {
        return Err(anyhow!("No host header"));
    };

    let remote_addr = {
        let mut addr = host.to_str()?.to_owned();
        if !addr.contains(':') {
            addr += ":80";
        }
        addr
    };

    tracing::debug!("Connecting to {}", remote_addr);
    let mut outbound = TcpStream::connect(remote_addr.clone()).await?;

    if request.method() == http::Method::CONNECT {
        inbound.write_all(CONNECTION_ESTABLISHED).await?;
        inbound.flush().await?;
    } else {
        outbound.write_all(&req_buffer).await?;
    }

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

async fn parse(
    stream: &mut (impl AsyncReadExt + Unpin + Send),
    req_buffer: &mut Vec<u8>,
) -> anyhow::Result<Request<()>> {
    let mut buf: Vec<u8> = vec![0; 4096];

    let req = loop {
        let size = stream.read(&mut buf).await?;
        tracing::debug!("Read {} bytes from tcp stream", size);
        req_buffer
            .extend(buf.get(..size).expect("shoud have at least this size"));

        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        let status = req.parse(req_buffer)?;

        match status {
            httparse::Status::Complete(body_start) => {
                tracing::debug!(
                    "Method: {:?} Path: {:?} Version: {:?} Body Start: {}",
                    req.method,
                    req.path,
                    req.version,
                    body_start,
                );

                if body_start != size {
                    return Err(anyhow!("body_start != size"));
                }

                let mut request = Request::builder();
                if let Some(version) = req.version {
                    let version = match version {
                        0 => http::Version::HTTP_10,
                        1 => http::Version::HTTP_11,
                        _ => return Err(anyhow!("Unknown HTTP version")),
                    };
                    request = request.version(version);
                };
                if let Some(method) = req.method {
                    request = request.method(method);
                }
                for header in req.headers {
                    request = request.header(header.name, header.value);
                }
                let request = request.body(())?;
                break request;
            },
            httparse::Status::Partial => {
                tracing::debug!("Partial request. Reading more.");
            },
        };
    };

    Ok(req)
}
