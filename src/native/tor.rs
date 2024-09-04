// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::PathBuf;
use std::sync::OnceLock;

use arti_client::config::{CfgPath, ConfigBuildError, TorClientConfigBuilder};
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

fn init_tor_client(custom_path: Option<PathBuf>) -> Result<TorClient<PreferredRuntime>, Error> {
    // Construct default Tor Client config
    let mut config = TorClientConfigBuilder::default();

    // Enable hidden services
    config.address_filter().allow_onion_addrs(true);

    // Check if is set a custom arti cache path
    if let Some(path) = custom_path {
        let cache: PathBuf = path.join("cache");
        let state: PathBuf = path.join("state");

        let cache_dir: CfgPath = CfgPath::new(cache.to_string_lossy().to_string());
        let state_dir: CfgPath = CfgPath::new(state.to_string_lossy().to_string());

        // Set custom paths
        config.storage().cache_dir(cache_dir).state_dir(state_dir);
    }

    let config: TorClientConfig = config.build()?;
    Ok(TorClient::builder()
        .config(config)
        .create_unbootstrapped()?)
}

/// Get or init tor client
#[inline]
fn get_tor_client<'a>(
    custom_path: Option<PathBuf>,
) -> Result<&'a TorClient<PreferredRuntime>, Error> {
    // TODO: replace with `get_or_try_init` when will be stable
    match TOR_CLIENT.get_or_init(|| init_tor_client(custom_path)) {
        Ok(client) => Ok(client),
        Err(e) => Err(e.clone()),
    }
}

#[inline]
pub(super) async fn connect(
    domain: &str,
    port: u16,
    custom_path: Option<PathBuf>,
) -> Result<DataStream, Error> {
    let client: &TorClient<PreferredRuntime> = get_tor_client(custom_path)?;
    Ok(client.connect((domain, port)).await?)
}
