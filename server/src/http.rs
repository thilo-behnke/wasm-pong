use std::convert::Infallible;
use std::fs::read;
use std::io::ErrorKind::NotFound;
use std::net::SocketAddr;
use std::sync::Arc;
use hyper::{Body, body, Method, Request, Response, Server, StatusCode};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper_tungstenite::HyperWebsocket;
use hyper_tungstenite::tungstenite::{Error, Message};
use kafka::producer::Producer;
use serde_json::json;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use pong::event::event::{Event, EventReader, EventWriter};
use futures::{sink::SinkExt, stream::StreamExt};
use crate::kafka::{KafkaEventReaderImpl, KafkaSessionEventWriterImpl};
use crate::player::Player;
use crate::session::{CachingSessionManager, SessionManager};
use crate::utils::http_utils::{get_query_params, read_json_body};

pub struct HttpServer {
    addr: [u8; 4],
    port: u16,
    session_manager: Arc<Mutex<CachingSessionManager>>
}
impl HttpServer {
    pub fn new(addr: [u8; 4], port: u16, kafka_host: &str) -> HttpServer {
        let session_manager = Arc::new(Mutex::new(CachingSessionManager::new(kafka_host)));
        HttpServer {addr, port, session_manager}
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>  {
        let make_svc = make_service_fn(|socket: &AddrStream| {
            let mut session_manager = Arc::clone(&self.session_manager);
            let addr = socket.remote_addr();
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let mut session_manager = Arc::clone(&session_manager);
                    async move {
                        if hyper_tungstenite::is_upgrade_request(&req) {
                            let (response, websocket) = hyper_tungstenite::upgrade(req, None).unwrap();

                            // Spawn a task to handle the websocket connection.
                            tokio::spawn(async move {
                                if let Err(e) = serve_websocket(websocket).await {
                                    eprintln!("Error in websocket connection: {}", e);
                                }
                            });

                            // Return the response so the spawned future can continue.
                            return Ok(response)
                        }

                        return handle_request(&session_manager, req, addr).await;
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

/// Handle a websocket connection.
async fn serve_websocket(websocket: HyperWebsocket) -> Result<(), Error> {
    let mut websocket = websocket.await?;
    while let Some(message) = websocket.next().await {
        match message? {
            Message::Text(msg) => {
                println!("Received text message: {}", msg);
                websocket.send(Message::text("Thank you, come again.")).await?;
            },
            Message::Binary(msg) => {
                println!("Received binary message: {:02X?}", msg);
                websocket.send(Message::binary(b"Thank you, come again.".to_vec())).await?;
            },
            Message::Ping(msg) => {
                // No need to send a reply: tungstenite takes care of this for you.
                println!("Received ping message: {:02X?}", msg);
            },
            Message::Pong(msg) => {
                println!("Received pong message: {:02X?}", msg);
            }
            Message::Close(msg) => {
                // No need to send a reply: tungstenite takes care of this for you.
                if let Some(msg) = &msg {
                    println!("Received close message with code {} and message: {}", msg.code, msg.reason);
                } else {
                    println!("Received close message");
                }
            },
            _ => {}
        }
    }

    Ok(())
}

// TODO: How to handle event writes/reads? This must be a websocket, but how to implement in hyper (if possible)?
// https://github.com/de-vri-es/hyper-tungstenite-rs
async fn handle_request(session_manager: &Arc<Mutex<CachingSessionManager>>, req: Request<Body>, addr: SocketAddr) -> Result<Response<Body>, Infallible> {
    println!("req to {} with method {}", req.uri().path(), req.method());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/session") => handle_get_session(session_manager, req).await,
        (&Method::POST, "/create_session") => handle_session_create(session_manager, req, addr).await,
        (&Method::POST, "/join_session") => handle_session_join(session_manager, req, addr).await,
        (&Method::POST, "/write") => handle_event_write(session_manager, req).await,
        (&Method::POST, "/read") => handle_event_read(session_manager, req).await,
        _ => Ok(Response::new("unknown".into()))
    }
}

async fn handle_get_session(session_manager: &Arc<Mutex<CachingSessionManager>>, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut locked = session_manager.lock().await;
    let query_params = get_query_params(&req);
    let session_id = query_params.get("session_id");
    if let None = session_id {
        return build_error_res("Please provide a valid session id", StatusCode::BAD_REQUEST);
    }
    let session_id = session_id.unwrap();
    let session = locked.get_session(session_id);
    if let None = session {
        return build_error_res("Unable to find session for given id", StatusCode::NOT_FOUND);
    }
    return build_success_res(&serde_json::to_string(&session.unwrap()).unwrap());
}

async fn handle_session_create(session_manager: &Arc<Mutex<CachingSessionManager>>, req: Request<Body>, addr: SocketAddr) -> Result<Response<Body>, Infallible> {
    let mut locked = session_manager.lock().await;
    let session_create_res = locked.create_session(Player {id: addr.to_string()}).await;
    if let Err(e) = session_create_res {
        return Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from(e)).unwrap());
    }
    let serialized = json!(session_create_res.unwrap());
    return Ok(Response::new(Body::from(serialized.to_string())))
}

async fn handle_session_join(session_manager: &Arc<Mutex<CachingSessionManager>>, mut req: Request<Body>, addr: SocketAddr) -> Result<Response<Body>, Infallible> {
    let mut locked = session_manager.lock().await;
    let body = read_json_body::<SessionJoinDto>(&mut req).await;
    let session_join_res = locked.join_session(body.session_id, Player {id: addr.to_string()}).await;
    if let Err(e) = session_join_res {
        return Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from(e)).unwrap());
    }
    let serialized = json!(session_join_res.unwrap());
    return Ok(Response::new(Body::from(serialized.to_string())))
}

async fn handle_event_write(session_manager: &Arc<Mutex<CachingSessionManager>>, mut req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut locked = session_manager.lock().await;
    let event = read_json_body::<SessionEventWriteDTO>(&mut req).await;
    let writer = locked.get_session_writer(&event.session_id);
    if let Err(e) = writer {
        let err = format!("Failed to write event: {}", e);
        println!("{}", err);
        let mut res = Response::new(Body::from(err));
        *res.status_mut() = StatusCode::NOT_FOUND;
        return Ok(res);
    }
    let mut writer = writer.unwrap();
    let mut writer_locked = writer.lock().await;
    println!("Writing session event to kafka: {:?}", event);
    let write_res = writer_locked.write_to_session(event.topic.clone(), event.msg.clone());
    if let Err(e) = write_res {
        let err = format!("Failed to write event: {}", e);
        println!("{}", err);
        let mut res = Response::new(Body::from(err));
        *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        return Ok(res);
    }
    println!("Successfully wrote event to kafka.");
    build_success_res(&serde_json::to_string(&event).unwrap())
}

async fn handle_event_read(session_manager: &Arc<Mutex<CachingSessionManager>>, mut req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut locked = session_manager.lock().await;
    let read_payload = read_json_body::<SessionReadDTO>(&mut req).await;
    let reader = locked.get_session_reader(&read_payload.session_id);
    if let Err(e) = reader {
        let err = format!("Failed to read events: {}", e);
        println!("{}", err);
        let mut res = Response::new(Body::from(err));
        *res.status_mut() = StatusCode::NOT_FOUND;
        return Ok(res);
    }
    let mut reader = reader.unwrap();
    let mut reader_locked = reader.lock().await;
    println!("Reading session events from kafka for session: {}", read_payload.session_id);
    let events = reader_locked.read_from_session();
    if let Err(e) = events {
        let err = format!("Failed to read events: {}", e);
        println!("{}", err);
        let mut res = Response::new(Body::from(err));
        *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        return Ok(res);
    }
    println!("Successfully read session events from kafka.");
    let json = serde_json::to_string(&events.unwrap()).unwrap();
    build_success_res(&json)
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

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

#[derive(Debug, Deserialize, Serialize)]
struct SessionEventWriteDTO {
    session_id: String,
    topic: String,
    msg: String
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionReadDTO {
    session_id: String
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionJoinDto {
    session_id: String
}
