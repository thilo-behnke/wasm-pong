use std::convert::Infallible;
use std::io::ErrorKind::NotFound;
use std::sync::Arc;
use hyper::{Body, body, Method, Request, Response, Server, StatusCode};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use kafka::producer::Producer;
use serde_json::json;
use tokio::sync::Mutex;
use pong::event::event::{Event, EventReader, EventWriter};
use crate::kafka::{KafkaEventReaderImpl, KafkaSessionEventWriterImpl};
use crate::session::SessionManager;
use crate::utils::http_utils::get_query_params;

pub struct HttpServer {
    addr: [u8; 4],
    port: u16,
    session_manager: Arc<Mutex<SessionManager>>,
    event_writer: Arc<Mutex<EventWriter>>,
    event_reader: Arc<Mutex<EventReader>>
}
impl HttpServer {
    pub fn new(addr: [u8; 4], port: u16, kafka_host: &str) -> HttpServer {
        let session_manager = Arc::new(Mutex::new(SessionManager::new(kafka_host)));
        let event_writer = Arc::new(Mutex::new(EventWriter::new(Box::new(KafkaSessionEventWriterImpl::session_writer(kafka_host)))));
        let event_reader = Arc::new(Mutex::new(EventReader::new(Box::new(KafkaEventReaderImpl::from(kafka_host)))));
        HttpServer {addr, port, session_manager, event_writer, event_reader}
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>  {
        let make_svc = make_service_fn(|socket: &AddrStream| {
            let mut session_manager = Arc::clone(&self.session_manager);
            let mut event_writer = Arc::clone(&self.event_writer);
            let mut event_reader = Arc::clone(&self.event_reader);
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let mut session_manager = Arc::clone(&session_manager);
                    let mut event_writer = Arc::clone(&event_writer);
                    let mut event_reader = Arc::clone(&event_reader);
                    async move {
                        return handle_request(&session_manager, &event_writer, &event_reader, req).await;
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

async fn handle_request(session_manager: &Arc<Mutex<SessionManager>>, event_writer: &Arc<Mutex<EventWriter>>, event_reader: &Arc<Mutex<EventReader>>, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("req to {} with method {}", req.uri().path(), req.method());
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/create_session") => handle_session_create(session_manager, req).await,
        (&Method::POST, "/write") => handle_event_write(event_writer, req).await,
        (&Method::GET, "/show") => handle_event_read(event_reader, req).await,
        _ => Ok(Response::new("unknown".into()))
    }
}

// TODO: Both for write and read session:
// - use session id from req body
// - pass event write / read to session manager that holds references to session specific readers/writers

async fn handle_session_create(session_manager: &Arc<Mutex<SessionManager>>, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut locked = session_manager.lock().await;
    let session_create_res = locked.create_session().await;
    if let Err(e) = session_create_res {
        return Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from(e)).unwrap());
    }
    let serialized = json!(session_create_res.unwrap());
    return Ok(Response::new(Body::from(serialized.to_string())))
}

async fn handle_event_write(event_writer: &Arc<Mutex<EventWriter>>, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut locked = event_writer.lock().await;
    let body = body::to_bytes(req.into_body()).await.unwrap();
    let event_str = std::str::from_utf8(&*body).unwrap();
    let event: Event = serde_json::from_str(event_str).unwrap();
    println!("Writing event to kafka: {:?}", event);
    if let Err(e) = locked.write(event) {
        println!("Failed to write to kafka! {:?}", e);
    }
    Ok(Response::new("response".into()))
}

async fn handle_event_read(event_reader: &Arc<Mutex<EventReader>>, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let query_params = get_query_params(&req);
    println!("{:?}", query_params);
    let topic = query_params.get("topic");
    let key = query_params.get("key");
    match (topic, key) {
        (Some(topic), Some(key)) => {
            let mut locked = event_reader.lock().await;
            let events = locked.read().unwrap();
            println!("read {} events", events.len());
            Ok(Response::new(format!("{:?}", events).into()))
        },
        _ => Ok(Response::new(format!("provide topic and key as query params").into())),
    }
}


async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
