// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::sync::OnceLock;

use arti_client::config::{ConfigBuildError, TorClientConfigBuilder};
use arti_client::{DataStream, TorClient, TorClientConfig};
use thiserror::Error;
use tor_rtcompat::PreferredRuntime;

static TOR_CLIENT: OnceLock<Result<TorClient<PreferredRuntime>, Error>> = OnceLock::new();

#[derive(Debug, Clone, Error)]
pub enum Error {
    /// Arti Client error
    #[error(transparent)]
    ArtiClient(#[from] arti_client::Error),
    /// Config builder error
    #[error(transparent)]
    ConfigBuilder(#[from] ConfigBuildError),
}

fn init_tor_client() -> Result<TorClient<PreferredRuntime>, Error> {
    let mut config = TorClientConfigBuilder::default();
    config.address_filter().allow_onion_addrs(true);
    let config: TorClientConfig = config.build()?;
    Ok(TorClient::builder()
        .config(config)
        .create_unbootstrapped()?)
}

/// Get or init tor client
#[inline]
fn get_tor_client<'a>() -> Result<&'a TorClient<PreferredRuntime>, Error> {
    // TODO: replace with `get_or_try_init` when will be stable
    match TOR_CLIENT.get_or_init(init_tor_client) {
        Ok(client) => Ok(client),
        Err(e) => Err(e.clone()),
    }
}

#[inline]
pub(super) async fn connect(domain: &str, port: u16) -> Result<DataStream, Error> {
    let client: &TorClient<PreferredRuntime> = get_tor_client()?;
    Ok(client.connect((domain, port)).await?)
}
