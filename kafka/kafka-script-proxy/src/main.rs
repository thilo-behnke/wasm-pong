extern crate core;

use std::collections::HashMap;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode, Uri};
use std::convert::Infallible;
use std::process::Output;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const TOPICS: [&str; 4] = ["host_tick", "peer_tick", "heart_beat", "session"];

#[tokio::main]
pub async fn main() {
    run().await.unwrap()
}

pub async fn run() -> Result<(), ()> {
    let make_svc = make_service_fn(|_| async {
        Ok::<_, Infallible>(service_fn(|req: Request<Body>| async {
            handle_request(req).await
        }))
    });

    let host = ([0, 0, 0, 0], 7243).into();
    let server = Server::bind(&host).serve(make_svc);
    write_to_log(&format!("Listening on http://{}", host)).await;
    let graceful = server.with_graceful_shutdown(shutdown_signal());
    graceful.await.unwrap();
    Ok(())
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    write_to_log(&format!(
        "req to {} with method {}",
        req.uri().path(),
        req.method()
    ))
    .await;
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/health_check") => handle_health_check().await,
        (&Method::POST, "/create_topic") => handle_create_topic(&req.uri()).await,
        (&Method::POST, "/add_partition") => handle_add_partition().await,
        _ => build_error_res("not found", StatusCode::NOT_FOUND),
    }
}

async fn handle_health_check() -> Result<Response<Body>, Infallible> {
    return match run_command(&format!("/opt/bitnami/kafka/bin/kafka-topics.sh --describe --bootstrap-server localhost:9093")).await {
        Ok(_) => {
            build_success_res("health check passed")
        },
        Err(e) => {
            write_to_log(&format!("health check failed: {:?}", e));
            build_error_res("health check failed", StatusCode::NOT_FOUND)
        }
    }
}

async fn handle_create_topic(uri: &Uri) -> Result<Response<Body>, Infallible> {
    let query_params = get_query_params(uri);
    let topic = query_params.get("topic");
    if let None = topic {
        return build_error_res("Missing mandatory query param >topic<", StatusCode::BAD_REQUEST);
    }
    let topic = topic.unwrap();

    write_to_log(&format!("Called to create topic {}.", topic)).await;
    if verify_topic_exists(topic).await {
        write_to_log(&format!("Topic {} already exists, noop.", topic)).await;
        return build_success_res("topic already exists");
    }

    write_to_log(&format!("Topic {} does not already exists, try to create it now.", topic)).await;

    run_command(&format!("/opt/bitnami/kafka/bin/kafka-topics.sh --create --topic {} --bootstrap-server localhost:9093", topic)).await
        .map(|_| build_success_res(&format!("successfully created topic {}", topic)))
        .map_err(|e| {
            build_error_res(&format!("failed to create topic {}", topic), StatusCode::INTERNAL_SERVER_ERROR)
        })
        .unwrap()
}

pub fn get_query_params(uri: &Uri) -> HashMap<&str, &str> {
    let query = uri.query();
    println!("uri={:?}, query={:?}", uri, query);
    match query {
        None => HashMap::new(),
        Some(query) => query
            .split("&")
            .map(|s| s.split_at(s.find("=").unwrap()))
            .map(|(key, value)| (key, &value[1..]))
            .collect(),
    }
}


async fn verify_topic_exists(topic: &str) -> bool {
    return match run_command(&format!("/opt/bitnami/kafka/bin/kafka-topics.sh --describe --topic {} --bootstrap-server localhost:9093", topic)).await {
        Ok(_) => {
            write_to_log(&format!("topic {} exists", topic)).await;
            true
        },
        Err(e) => {
            write_to_log(&format!("topic {} does not exist or caused other issues: {:?}", topic, e)).await;
            false
        }
    }
}

async fn handle_add_partition() -> Result<Response<Body>, Infallible> {
    write_to_log("Called to add partition.").await;
    let current_count = get_highest_partition_count().await;
    if let Err(_) = current_count {
        let err = "Failed to retrieve max partition count.";
        write_to_log(err).await;
        return build_error_res(err, StatusCode::INTERNAL_SERVER_ERROR);
    }
    let current_count = current_count.unwrap();
    write_to_log(&format!("Successfully retrieved current max partition count: {}.", current_count)).await;

    let next_partition = current_count + 1;
    write_to_log(&format!("Updating partition count to {} for the following topics: {}", next_partition, TOPICS.join(","))).await;
    for topic in TOPICS {
        let output = run_command(&format!("/opt/bitnami/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9093 --alter --topic {} --partitions {}", topic, next_partition)).await;
        if let Err(e) = output {
            let error = format!("Failed to update the partition count: {}", e);
            write_to_log(&error).await;
            return build_error_res(&error, StatusCode::INTERNAL_SERVER_ERROR);
        }
        let output = output.unwrap();
        if !output.status.success() {
            let error = format!("Failed to update the partition count: {:?}", std::str::from_utf8(&*output.stderr).unwrap());
            write_to_log(&error).await;
            return build_error_res(&error, StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    write_to_log(&format!("Successfully updated partition count to {}", next_partition)).await;
    return build_success_res(&current_count.to_string()); // current_count because count - 1 = max partition
}

async fn get_highest_partition_count() -> Result<u32, PartitionCountQueryError> {
    let output = run_command("/opt/bitnami/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9093 --describe | grep -Po '(?<=PartitionCount: )(\\d+)' | sort -r | head -1").await.unwrap();
    let output_str = std::str::from_utf8(&*output.stdout);
    if let Err(e) = output_str {
        let message = format!("Failed to convert command output to string: {:?}", e);
        write_to_log(&message).await;
        return Err(PartitionCountQueryError { message });
    }

    let output_str = output_str.unwrap().trim().replace("\n", "");
    let parse_res = output_str.parse::<u32>();
    if let Err(e) = parse_res {
        let message = format!("Failed to parse partition count for output {}: {:?}", output_str, e);
        write_to_log(&message).await;
        return Err(PartitionCountQueryError {message})
    }
    return Ok(parse_res.unwrap())
}

#[derive(Debug)]
struct PartitionCountQueryError {
    message: String
}

async fn run_command(command: &str) -> Result<Output, String> {
    write_to_log(&format!("Running command: {}", command)).await;
    let output = Command::new("/bin/bash")
        .arg("-c")
        .arg(command)
        .output()
        .await.unwrap();
    let stdout = std::str::from_utf8(&*output.stdout).unwrap();
    let stderr = std::str::from_utf8(&*output.stderr).unwrap();
    write_to_log(&format!("Command returned stdout: {}", stdout)).await;
    write_to_log(&format!("Command returned stderr: {}", stderr)).await;
    return match output.status.success() {
        true => Ok(output),
        false => Err(stderr.to_owned())
    };
}

pub fn build_success_res(value: &str) -> Result<Response<Body>, Infallible> {
    let json = format!("{{\"data\": {}}}", value);
    return Ok(Response::new(Body::from(json)));
}

pub fn build_error_res(error: &str, status: StatusCode) -> Result<Response<Body>, Infallible> {
    let json = format!("{{\"error\": \"{}\"}}", error);
    let mut res = Response::new(Body::from(json));
    *res.status_mut() = status;
    return Ok(res);
}

pub async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

async fn write_to_log(value: &str) {
    let content = format!("{}\n", value);
    let mut options = OpenOptions::new();
    let file = options
        .create(true)
        .write(true)
        .append(true)
        .open("/var/log/kafka-script-proxy/log.txt")
        .await;
    if let Err(e) = file {
        println!("Failed to open file: {}", e);
        return;
    }
    let write_res = file.unwrap().write_all(content.as_bytes()).await;
    if let Err(e) = write_res {
        println!("Failed to write to file: {:?}", e);
    }
}
