extern crate core;

use hyper::{Body, Client, Method, Request};
use log::{debug, error, info, Level, LevelFilter};
use log4rs::append::console::{ConsoleAppender, ConsoleAppenderConfig};
use log4rs::append::file::FileAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::delete::DeleteRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
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
    let compound_policy = Box::new(CompoundPolicy::new(Box::new(SizeTrigger::new(2u64.pow(20) * 10)), Box::new(DeleteRoller::new())));
    let logfile = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S%.f)} [{l}] - [{M}:{L}] {m}\n")))
        .append(true)
        .build("log/output.log", compound_policy).unwrap();

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}] {m}\n")))
        .build();

    let config = Config::builder()
        .appenders(
            vec![
                Appender::builder().build("stdout", Box::new(stdout)),
                Appender::builder().build("logfile", Box::new(logfile)),
            ]
        )
        .logger(
            Logger::builder()
                .appender("logfile")
                .additive(true)
                .build("server", LevelFilter::Trace)
        )
        .build(Root::builder()
            .appender("stdout")
            .build(LevelFilter::Info)).unwrap();

    let logger_res = log4rs::init_config(config);
    if let Err(e) = logger_res {
        eprintln!("Failed to setup logger: {:?}", e);
        return;
    }

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

    info!("BOOTSTRAP: Create topics if needed");
    let topics = ["session", "host_tick", "peer_tick", "heart_beat"];
    let http_client = Client::new();
    for topic in topics {
        let req = Request::builder().method(Method::POST).uri(format!("{}/create_topic?topic={}", kafka_topic_manager_host, topic)).body(Body::empty()).expect("request builder for topic creation");
        let res = http_client.request(req).await;
        match res {
            Ok(_) => {
                info!("Successfully created topic {}", topic);
            },
            Err(e) => {
                error!("Failed to create topic {}: {:?}", topic, e);
            }
        }
    }

    info!("booting up server");
    HttpServer::new([0, 0, 0, 0], 4000, &kafka_host, &kafka_topic_manager_host)
        .run()
        .await
        .expect("failed to run server");
}
