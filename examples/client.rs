use std::time::Duration;

use async_wsocket::{ConnectionMode, Url};

#[tokio::main]
async fn main() {
    let url =
        Url::parse("ws://2jsnlhfnelig5acq6iacydmzdbdmg7xwunm4xl6qwbvzacw4lwrjmlyd.onion").unwrap();
    let (_tx, _rx) = async_wsocket::connect(&url, ConnectionMode::Tor, Duration::from_secs(120))
        .await
        .unwrap();
}
