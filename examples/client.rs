use url::Url;

#[tokio::main]
async fn main() {
    let url = Url::parse("wss://relay.mostro.network").unwrap();
    let (_tx, _rx) = async_wsocket::connect(&url, None, None).await.unwrap();
}
