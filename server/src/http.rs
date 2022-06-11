use std::convert::Infallible;
use std::fs::read;
use std::io::ErrorKind::NotFound;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use hyper::{Body, body, Method, Request, Response, Server, StatusCode};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper_tungstenite::{HyperWebsocket, tungstenite};
use hyper_tungstenite::tungstenite::{Error, Message};
use kafka::producer::Producer;
use serde_json::json;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use pong::event::event::{Event, EventReader, EventWriter};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::io::Sink;
use tokio::task;
use tokio::time::sleep;
use crate::kafka::{KafkaEventReaderImpl, KafkaSessionEventWriterImpl};
use crate::player::Player;
use crate::session::{Session, SessionManager};
use crate::utils::http_utils::{get_query_params, read_json_body};
use crate::utils::time_utils::now;

pub struct HttpServer {
    addr: [u8; 4],
    port: u16,
    session_manager: Arc<Mutex<SessionManager>>
}
impl HttpServer {
    pub fn new(addr: [u8; 4], port: u16, kafka_host: &str) -> HttpServer {
        let session_manager = Arc::new(Mutex::new(SessionManager::new(kafka_host)));
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
                            println!("Received request to upgrade to websocket connection: {:?}", req);
                            let params = get_query_params(&req);
                            println!("Ws request params: {:?}", params);
                            if !params.contains_key("session_id") {
                                eprintln!("Missing session id request param for websocket connection, don't upgrade connection to ws.");
                                return build_error_res("Missing request param: session_id", StatusCode::BAD_REQUEST);
                            }
                            if !params.contains_key("connection_type") {
                                eprintln!("Missing connection type request param for websocket connection, don't upgrade connection to ws.");
                                let res = build_error_res("Missing request param: connection_type", StatusCode::BAD_REQUEST);
                                return res;
                            }
                            let session_id = params.get("session_id").unwrap();
                            let connection_type_raw = params.get("connection_type").unwrap();
                            let connection_type = WebSocketConnectionType::from_str(connection_type_raw);
                            if let Err(_) = connection_type {
                                let error = format!("Invalid connection type: {}", connection_type_raw);
                                eprintln!("{}", error);
                                return build_error_res(error.as_str(), StatusCode::BAD_REQUEST);
                            }
                            let session = session_manager.lock().await.get_session(session_id);
                            if let None = session {
                                let error = format!("Session does not exist: {}", session_id);
                                eprintln!("{}", error);
                                return build_error_res(error.as_str(), StatusCode::NOT_FOUND);
                            }
                            let session = session.unwrap();
                            let websocket_session = WebSocketSession {session: session.clone(), connection_type: connection_type.unwrap()};
                            println!("Websocket upgrade request is valid, will now upgrade to websocket: {:?}", req);

                            let (response, websocket) = hyper_tungstenite::upgrade(req, None).unwrap();

                            // Spawn a task to handle the websocket connection.
                            tokio::spawn(async move {
                                if let Err(e) = serve_websocket(websocket_session, websocket, session_manager).await {
                                    eprintln!("Error in websocket connection: {:?}", e);
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
async fn serve_websocket(websocket_session: WebSocketSession, websocket: HyperWebsocket, session_manager: Arc<Mutex<SessionManager>>) -> Result<(), Error> {
    let mut websocket = websocket.await?;
    let (mut websocket_writer, mut websocket_reader) = websocket.split();

    let session_manager = session_manager.lock().await;
    let session = session_manager.get_session(&websocket_session.session.hash);
    let event_handler_pair = session_manager.split(&websocket_session.session.hash, websocket_session.connection_type.get_topics());
    if let Err(_) = event_handler_pair {
        eprintln!("Failed to create event reader/writer pair session: {:?}", websocket_session);
        return Err(Error::ConnectionClosed) // TODO: Use proper error for this case to close the connection
    }

    let (mut event_reader, mut event_writer) = event_handler_pair.unwrap();
    let websocket_session_read_copy = websocket_session.clone();
    tokio::spawn(async move {
        println!("Ready to read messages from ws connection: {:?}", websocket_session_read_copy);
        while let Some(message) = websocket_reader.next().await {
            match message.unwrap() {
                Message::Text(msg) => {
                    let events = serde_json::from_str::<SessionEventListDTO>(&msg);
                    println!("Received ws message to persist events to kafka");
                    if let Err(e) = events {
                        eprintln!("Failed to deserialize ws message to event {}: {}", msg, e);
                        continue;
                    }
                    let event_wrapper = events.unwrap();
                    if event_wrapper.session_id != websocket_session_read_copy.session.hash {
                        eprintln!("Websocket has session {:?} but was asked to write to session {} - skip.", websocket_session_read_copy, event_wrapper.session_id);
                        continue;
                    }
                    let mut any_error = false;
                    let event_count = event_wrapper.events.len();
                    for event in event_wrapper.events {
                        let write_res = event_writer.write_to_session(&event.topic, &event.msg);
                        if let Err(e) = write_res {
                            any_error = true;
                            eprintln!("Failed to write event {:?}: {}", event, e);
                        }
                    }
                    if any_error {
                        eprintln!("Failed to write at least one message for session {}", event_wrapper.session_id);
                    } else {
                        println!("Successfully wrote {} messages to kafka for session {:?}", event_count, websocket_session_read_copy)
                    }
                },
                Message::Close(msg) => {
                    // No need to send a reply: tungstenite takes care of this for you.
                    if let Some(msg) = &msg {
                        println!("Received close message with code {} and message: {}", msg.code, msg.reason);
                    } else {
                        println!("Received close message");
                    }

                    let session_closed_event = SessionClosedDto {
                        session: websocket_session_read_copy.session.clone(),
                        reason: "ws closed".to_owned()
                    };
                    let msg = json!(session_closed_event).to_string();
                    let session_event_write_res = event_writer.write_to_session("session", &msg);
                    if let Err(e) = session_event_write_res {
                        eprintln!("Failed to write session closed event: {0}", e)
                    }
                    break;
                },
                _ => {}
            }
        }
        println!("!!!! Exit websocket receiver !!!!")
    });
    let websocket_session_write_copy = websocket_session.clone();
    tokio::spawn(async move {
        println!("Ready to read messages from kafka: {:?}", websocket_session_write_copy);
        loop {
            println!("Reading messages from kafka.");
            let messages = event_reader.read_from_session();
            if let Err(_) = messages {
                eprintln!("Failed to read messages from kafka for session: {:?}", websocket_session_write_copy);
                continue;
            }
            // println!("Read messages for websocket_session {:?} from consumer: {:?}", websocket_session_write_copy, messages);
            let messages = messages.unwrap();
            if messages.len() == 0 {
                println!("No new messages from kafka.");
                continue;
            }
            println!("{} new messages from kafka.", messages.len());
            let json = serde_json::to_string(&messages).unwrap();
            let message = Message::from(json);
            println!("Sending kafka messages through websocket.");
            let send_res = websocket_writer.send(message).await;
            if let Err(e) = send_res {
                eprintln!("Failed to send message to websocket for session {:?}: {:?}", websocket_session_write_copy, e);
                match e {
                    tungstenite::error::Error::ConnectionClosed | tungstenite::error::Error::AlreadyClosed => {
                        println!("Websocket Connection for session {:?} is closed. Exiting kafka consumer.", websocket_session_write_copy);
                        break;
                    },
                    _ => {}
                }
            }
            // Avoid starvation of read thread (?)
            // TODO: How to avoid this? This is very bad for performance.
            sleep(Duration::from_millis(100)).await;
        }
    });
    Ok(())
}

// TODO: How to handle event writes/reads? This must be a websocket, but how to implement in hyper (if possible)?
// https://github.com/de-vri-es/hyper-tungstenite-rs
async fn handle_request(session_manager: &Arc<Mutex<SessionManager>>, req: Request<Body>, addr: SocketAddr) -> Result<Response<Body>, Infallible> {
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

async fn handle_get_session(session_manager: &Arc<Mutex<SessionManager>>, req: Request<Body>) -> Result<Response<Body>, Infallible> {
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

async fn handle_session_create(session_manager: &Arc<Mutex<SessionManager>>, req: Request<Body>, addr: SocketAddr) -> Result<Response<Body>, Infallible> {
    println!("Called to create new session: {:?}", req);
    let mut locked = session_manager.lock().await;
    let player = Player {id: addr.to_string()};
    let session_create_res = locked.create_session(player.clone()).await;
    if let Err(e) = session_create_res {
        return Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from(e)).unwrap());
    }
    let session_created = SessionCreatedDto {session: session_create_res.unwrap(), player};
    let serialized = json!(session_created);
    return build_success_res(&serialized.to_string());
}

async fn handle_session_join(session_manager: &Arc<Mutex<SessionManager>>, mut req: Request<Body>, addr: SocketAddr) -> Result<Response<Body>, Infallible> {
    println!("Received request to join session: {:?}", req);
    let mut locked = session_manager.lock().await;
    let body = read_json_body::<SessionJoinDto>(&mut req).await;
    let player = Player {id: addr.to_string()};
    let session_join_res = locked.join_session(body.session_id, player.clone()).await;
    if let Err(e) = session_join_res {
        eprintln!("Failed to join session: {:?}", e);
        return Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from(e)).unwrap());
    }
    let session = session_join_res.unwrap();
    println!("Successfully joined session: {:?}", session);
    let session_joined = SessionJoinedDto {session, player};
    let serialized = json!(session_joined);
    return build_success_res(&serialized.to_string());
}

async fn handle_event_write(session_manager: &Arc<Mutex<SessionManager>>, mut req: Request<Body>) -> Result<Response<Body>, Infallible> {
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
    println!("Writing session event to kafka: {:?}", event);
    let write_res = writer.write_to_session(&event.topic, &event.msg);
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

async fn handle_event_read(session_manager: &Arc<Mutex<SessionManager>>, mut req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut locked = session_manager.lock().await;
    let read_payload = read_json_body::<SessionReadDTO>(&mut req).await;
    let reader = locked.get_session_reader(&read_payload.session_id, &["move", "status", "input", "session"]);
    if let Err(e) = reader {
        let err = format!("Failed to read events: {}", e);
        println!("{}", err);
        let mut res = Response::new(Body::from(err));
        *res.status_mut() = StatusCode::NOT_FOUND;
        return Ok(res);
    }
    let mut reader = reader.unwrap();
    println!("Reading session events from kafka for session: {}", read_payload.session_id);
    let events = reader.read_from_session();
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
    let mut res = Response::new(Body::from(json));
    let headers = res.headers_mut();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Access-Control-Allow-Origin", "http://localhost:8080".parse().unwrap());
    Ok(res)
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
struct SessionEventListDTO {
    session_id: String,
    events: Vec<SessionEventWriteDTO>
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

#[derive(Debug, Serialize)]
struct SessionJoinedDto {
    session: Session,
    player: Player,
}

#[derive(Debug, Serialize)]
struct SessionCreatedDto {
    session: Session,
    player: Player,
}

#[derive(Debug, Serialize)]
struct SessionClosedDto {
    session: Session,
    reason: String
}

#[derive(Debug, Clone, PartialEq)]
enum WebSocketConnectionType {
    HOST, PEER, OBSERVER
}

impl FromStr for WebSocketConnectionType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "host" => Ok(WebSocketConnectionType::HOST),
            "peer" => Ok(WebSocketConnectionType::PEER),
            "observer" => Ok(WebSocketConnectionType::OBSERVER),
            _ => Err(())
        }
    }
}

#[derive(Debug, Clone)]
struct WebSocketSession {
    pub connection_type: WebSocketConnectionType,
    pub session: Session
}

impl WebSocketConnectionType {
    pub fn get_topics(&self) -> &[&str] {
        match self {
            WebSocketConnectionType::HOST => &["input", "session"],
            WebSocketConnectionType::PEER | WebSocketConnectionType::OBSERVER => &["move", "input", "status", "session"],
        }
    }
}
