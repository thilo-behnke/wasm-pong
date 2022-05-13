use std::convert::Infallible;
use std::io::ErrorKind::NotFound;
use std::sync::Arc;
use hyper::{Body, Method, Request, Response, Server};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use kafka::producer::Producer;
use tokio::sync::Mutex;
use pong::event::event::{Event, EventWriter};
use crate::kafka::KafkaEventWriterImpl;

pub struct HttpServer {
    addr: [u8; 4],
    port: u16,
    event_writer: Arc<Mutex<EventWriter>>
}
impl HttpServer {
    pub fn new(addr: [u8; 4], port: u16) -> HttpServer {
        let event_writer = Arc::new(Mutex::new(EventWriter::new(Box::new(KafkaEventWriterImpl::default()))));
        HttpServer {addr, port, event_writer}
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>  {
        let make_svc = make_service_fn(|socket: &AddrStream| {
            let mut event_writer = Arc::clone(&self.event_writer);
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let mut event_writer = Arc::clone(&event_writer);
                    async move {
                        return handle_request(&event_writer, req).await;
                    }
                }))
            }
        });

        let host = (self.addr, self.port).into();
        let server = Server::bind(&host).serve(make_svc);
        println!("Listening on http://{}", host);
        let graceful = server.with_graceful_shutdown(shutdown_signal());
        graceful.await?;
        Ok(())
    }
}

async fn handle_request(event_writer: &Arc<Mutex<EventWriter>>, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("req to {} with method {}", req.uri().path(), req.method());
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/write") => handle_event_write(event_writer, req).await,
        (&Method::GET, "/show") => handle_event_read(req).await,
        _ => Ok(Response::new("unknown".into()))
    }
}

async fn handle_event_write(event_writer: &Arc<Mutex<EventWriter>>, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut locked = event_writer.lock().await;
    let event = Event {topic: "topic".into(), key: "key".into(), msg: "msg".into()};
    println!("Writing event to kafka: {:?}", event);
    if let Err(e) = locked.write(event) {
        println!("Failed to write to kafka! {:?}", e);
    }
    Ok(Response::new("response".into()))
}

async fn handle_event_read(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("events".into()))
}


async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
