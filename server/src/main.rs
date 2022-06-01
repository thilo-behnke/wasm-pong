extern crate core;

use crate::http::HttpServer;

pub mod http;
pub mod kafka;
pub mod utils;
mod hash;
mod session;
mod player;

#[tokio::main]
pub async fn main() {
    HttpServer::new([127, 0, 0, 1], 4000, "localhost:9093").run().await.expect("failed to run server");
}
