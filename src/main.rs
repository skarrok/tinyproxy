mod config;
mod logger;
mod proxy;

use clap::Parser;
use dotenvy::dotenv;
use tokio::net::TcpListener;

use config::Config;
use config::LogStruct;
use proxy::http;
use proxy::socks5;
use proxy::tcp;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let config = Config::parse();

    logger::setup(
        config.log_level,
        config.log_format,
        option_env!("CARGO_BIN_NAME"),
    );

    config.log();

    let listener = TcpListener::bind(config.listen_address).await?;

    match config.proxy_mode {
        config::Mode::Http => http::handle(listener).await,
        config::Mode::Socks5 => socks5::handle(listener).await,
        config::Mode::Tcp => {
            tcp::handle(
                listener,
                config.remote_address.expect("should be set by CLI"),
            )
            .await;
        },
    };

    Ok(())
}
