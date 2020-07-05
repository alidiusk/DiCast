use warp::Filter;

use std::net::SocketAddr;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    let hello = warp::path!("hello" / String)
        .map(|name| {
            log::info!("Received a request: /hello/{}", name);

            format!("Hello, {}!", name)
        });

    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    warp::serve(hello)
        .run(addr)
        .await;

    Ok(())
}
