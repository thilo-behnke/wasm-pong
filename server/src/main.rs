extern crate core;

use crate::http::HttpServer;

mod hash;
pub mod http;
pub mod kafka;
mod player;
mod session;
pub mod utils;

#[tokio::main]
pub async fn main() {
    let kafka_host_env = std::env::var("KAFKA_HOST");
    let kafka_host = match kafka_host_env {
        Ok(val) => val,
        Err(_) => "localhost:9093".to_owned()
    };

    HttpServer::new([0, 0, 0, 0], 4000, &kafka_host)
        .run()
        .await
        .expect("failed to run server");
}
