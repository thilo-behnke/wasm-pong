use crate::http::HttpServer;

mod http;
mod kafka;

#[tokio::main]
pub async fn main() {
    HttpServer::new([127, 0, 0, 1], 4000).run().await.expect("failed to run server");
}
