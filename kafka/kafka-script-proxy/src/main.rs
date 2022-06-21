extern crate core;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::process::Output;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const TOPICS: [&str; 5] = ["move", "status", "input", "heart_beat", "session"];

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
        (&Method::POST, "/add_partition") => handle_add_partition().await,
        _ => build_error_res("not found", StatusCode::NOT_FOUND),
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

async fn run_command(command: &str) -> std::io::Result<Output> {
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
    Ok(output)
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
