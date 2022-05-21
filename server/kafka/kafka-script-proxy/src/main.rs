use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use std::convert::Infallible;
use std::fs::write;
use std::process::Output;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const TOPICS: [&str; 3] = ["move", "status", "input"];

#[tokio::main]
pub async fn main() {
    run().await.unwrap()
}

pub async fn run() -> Result<(), ()> {
    let make_svc = make_service_fn(|socket: &AddrStream| async {
        Ok::<_, Infallible>(service_fn(|req: Request<Body>| async {
            handle_request(req).await
        }))
    });

    let host = ([0, 0, 0, 0], 7243).into();
    let server = Server::bind(&host).serve(make_svc);
    write_to_log(&format!("Listening on http://{}", host)).await;
    let graceful = server.with_graceful_shutdown(shutdown_signal());
    graceful.await;
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
        _ => Ok(Response::new("unknown".into())),
    }
}

async fn handle_add_partition() -> Result<Response<Body>, Infallible> {
    let current_count = get_highest_partition_count().await;
    let next_partition = current_count + 1;
    for topic in TOPICS {
        let output = run_command(&format!("/opt/bitnami/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --alter --topic {} --partitions {}", topic, next_partition)).await;
        if let Err(e) = output {
            write_to_log(&format!("{}", e)).await;
            return Ok(Response::new(Body::empty()));
        }
        let output = output.unwrap();
        if !output.status.success() {
            write_to_log(&format!("{:?}", std::str::from_utf8(&*output.stderr))).await;
            return Ok(Response::new(Body::empty()));
        }
    }
    Ok(Response::new(Body::from(next_partition.to_string())))
}

async fn get_highest_partition_count() -> u32 {
    let output = run_command("/opt/bitnami/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --describe | grep -Po '(?<=PartitionCount: )(\\d+)' | sort -r | head -1").await.unwrap();
    let stdout = std::str::from_utf8(&*output.stdout).unwrap().to_owned();
    stdout.parse::<u32>().unwrap()
}

async fn run_command(command: &str) -> std::io::Result<Output> {
    write_to_log(&format!("Running command: {}", command)).await;
    let output = Command::new("/bin/bash")
        .arg(format!("-c {}", command))
        .output()
        .await.unwrap();
    let stdout = std::str::from_utf8(&*output.stdout).unwrap();
    let stderr = std::str::from_utf8(&*output.stderr).unwrap();
    write_to_log(&format!("Command returned stdout: {}", stdout));
    write_to_log(&format!("Command returned stdout: {}", stderr));
    Ok(output)
}

pub async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

async fn write_to_log(value: &str) -> std::io::Result<()> {
    let content = format!("{}\n", value);
    let mut options = OpenOptions::new();
    let mut file = options
        .create(true)
        .write(true)
        .append(true)
        .open("/var/lib/kafka-script-proxy/log.txt")
        .await;
    if let Err(e) = file {
        println!("Failed to open file: {}", e);
        return Ok(());
    }
    file.unwrap().write_all(content.as_bytes()).await
}
