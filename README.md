# Async WebSocket

A convenience library for using websockets both in native and WASM environments! Include embedded tor client support.

```rust
use std::time::Duration;

use async_wsocket::{ConnectionMode, Url};

#[tokio::main]
async fn main() {
    let url = Url::parse("wss://example.com").unwrap();
    // Use `ConnectionMode::Tor` to use the embedded tor client (require `tor` feature)
    let (_tx, _rx) = async_wsocket::connect(&url, ConnectionMode::Direct, Duration::from_secs(120))
        .await
        .unwrap();
}
```

## Crate Feature Flags

The following crate feature flags are available:

| Feature | Default | Description                        |
|---------|:-------:|------------------------------------|
| `socks` |   No    | Enable `socks` proxy support       |
| `tor`   |   No    | Enable embedded tor client support |

## Minimum Supported Rust Version (MSRV)

The MSRV for this project when compiled with `default` features and on `native` targets is `1.63.0`. 
When using `tor` feature, MSRV is `1.70.0`. MSRV for WASM targets is `1.73.0`

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details

## Donations

⚡ Tips: <https://getalby.com/p/yuki>

⚡ Lightning Address: yuki@getalby.com