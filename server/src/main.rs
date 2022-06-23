extern crate core;

use log::{debug, error, info, Level};
use crate::http::HttpServer;

mod hash;
pub mod http;
pub mod kafka;
mod session_manager;
pub mod utils;
mod websocket_handler;
mod request_handler;
mod event;
mod actor;
mod session;

#[tokio::main]
pub async fn main() {
    env_logger::init();
    info!("preparing environment");
    let kafka_host_env = std::env::var("KAFKA_HOST");
    let kafka_host = match kafka_host_env {
        Ok(val) => val,
        Err(_) => "localhost:9093".to_owned(),
    };
    info!("KAFKA_HOST={}", kafka_host);
    let kafka_partition_manager_host_env = std::env::var("KAFKA_TOPIC_MANAGER_HOST");
    let kafka_topic_manager_host = match kafka_partition_manager_host_env {
        Ok(val) => val,
        Err(_) => "localhost:7243".to_owned(),
    };
    info!("KAFKA_TOPIC_MANAGER_HOST={}", kafka_topic_manager_host);

    info!("booting up server");
    HttpServer::new([0, 0, 0, 0], 4000, &kafka_host, &kafka_topic_manager_host)
        .run()
        .await
        .expect("failed to run server");
}
