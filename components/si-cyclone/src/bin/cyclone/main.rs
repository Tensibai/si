use std::convert::TryInto;

use color_eyre::Result;
use si_cyclone::{telemetry, Config, IncomingStream, Server};

mod args;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    telemetry::init()?;
    let config = args::parse().try_into()?;

    run(config).await
}

async fn run(config: Config) -> Result<()> {
    match config.incoming_stream() {
        IncomingStream::HTTPSocket(_) => Server::http(config)?.run().await?,
        IncomingStream::UnixDomainSocket(_) => Server::uds(config).await?.run().await?,
    }

    Ok(())
}
