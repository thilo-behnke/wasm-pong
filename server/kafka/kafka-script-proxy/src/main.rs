use std::convert::Infallible;
use hyper::{Body, Method, Request, Response, Server};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
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

    let host = ([0,0,0,0], 7243).into();
    let server = Server::bind(&host).serve(make_svc);
    println!("Listening on http://{}", host);
    let graceful = server.with_graceful_shutdown(shutdown_signal());
    graceful.await;
    Ok(())
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("req to {} with method {}", req.uri().path(), req.method());
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/add_partition") => handle_add_partition().await,
        _ => Ok(Response::new("unknown".into()))
    }
}

async fn handle_add_partition() -> Result<Response<Body>, Infallible>  {
    let current_count = get_highest_partition_count().await;
    let next_partition = current_count + 1;
    for topic in TOPICS {
        let output = Command::new("/bin/bash").arg(format!("-c /opt/bitnami/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --alter --topic {} --partitions {}", topic, next_partition)).output().await;
        if let Err(e) = output {
            println!("{}", e);
            return Ok(Response::new(Body::empty()));
        }
        let output = output.unwrap();
        if !output.status.success() {
            println!("{:?}", std::str::from_utf8(&*output.stderr));
            return Ok(Response::new(Body::empty()));
        }
    }
    Ok(Response::new(Body::from(next_partition.to_string())))
}

async fn get_highest_partition_count() -> u32 {
    let output = Command::new("/bin/bash").arg("-c /opt/bitnami/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --describe | grep -Po '(?<=Partition: )(\\d+)' | sort -r | head -1").output().await.unwrap();
    let stdout = std::str::from_utf8(&*output.stdout).unwrap().to_owned();
    stdout.parse::<u32>().unwrap()
}

pub async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
