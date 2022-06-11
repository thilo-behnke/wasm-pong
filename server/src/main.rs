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
    HttpServer::new([192, 168, 178, 0], 4000, "localhost:9093")
        .run()
        .await
        .expect("failed to run server");
}
