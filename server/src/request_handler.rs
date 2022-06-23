use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use hyper::{Body, Method, Request, Response, StatusCode};
use async_trait::async_trait;
use serde_json::json;
use tokio::sync::Mutex;
use serde::{Deserialize};
use crate::event::{SessionEvent, SessionEventPayload, SessionEventType};
use crate::actor::Player;
use crate::session_manager::SessionManager;
use crate::utils::http_utils::{build_error_res, build_success_res, get_query_params, read_json_body};

#[async_trait]
pub trait RequestHandler {
    async fn handle(&self, req: Request<Body>, addr: SocketAddr) -> Result<Response<Body>, Infallible>;
}

pub struct DefaultRequestHandler {
    session_manager: Arc<Mutex<SessionManager>>
}

impl DefaultRequestHandler {
    pub fn new(
        session_manager: Arc<Mutex<SessionManager>>
    ) -> DefaultRequestHandler {
        DefaultRequestHandler {
            session_manager
        }
    }
}

#[async_trait]
impl RequestHandler for DefaultRequestHandler {
    async fn handle(&self, req: Request<Body>, addr: SocketAddr) -> Result<Response<Body>, Infallible>  {
        println!("req to {} with method {}", req.uri().path(), req.method());
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/session") => handle_get_session(&self.session_manager, req).await,
            (&Method::POST, "/create_session") => {
                handle_session_create(&self.session_manager, req, addr).await
            }
            (&Method::POST, "/join_session") => handle_session_join(&self.session_manager, req, addr).await,
            _ => Ok(Response::new("unknown".into())),
        }
    }
    
}

async fn handle_get_session(
    session_manager: &Arc<Mutex<SessionManager>>,
    req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    let locked = session_manager.lock().await;
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

async fn handle_session_create(
    session_manager: &Arc<Mutex<SessionManager>>,
    req: Request<Body>,
    addr: SocketAddr,
) -> Result<Response<Body>, Infallible> {
    println!("Called to create new session: {:?}", req);
    let mut locked = session_manager.lock().await;
    let player = Player::new(1, addr.ip().to_string());
    let session_create_res = locked.create_session(player.clone()).await;
    if let Err(e) = session_create_res {
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(e))
            .unwrap());
    }
    let reason = format!("player {:?} created session", player);
    let session_created = SessionEvent::Created(SessionEventPayload {
        session: session_create_res.unwrap(),
        actor: player,
        reason
    });
    let serialized = json!(session_created);
    return build_success_res(&serialized.to_string());
}

async fn handle_session_join(
    session_manager: &Arc<Mutex<SessionManager>>,
    mut req: Request<Body>,
    addr: SocketAddr,
) -> Result<Response<Body>, Infallible> {
    println!("Received request to join session: {:?}", req);
    let mut locked = session_manager.lock().await;
    let body = read_json_body::<SessionJoinDto>(&mut req).await;
    let player = Player::new(2, addr.ip().to_string());
    let session_join_res = locked.join_session(body.session_id, player.clone()).await;
    if let Err(e) = session_join_res {
        eprintln!("Failed to join session: {:?}", e);
        return Ok(Response::builder()
            .status(StatusCode::CONFLICT)
            .body(Body::from(e))
            .unwrap());
    }
    let session = session_join_res.unwrap();
    println!("Successfully joined session: {:?}", session);
    let reason = format!("player {:?} joined session", player);
    let session_joined = SessionEvent::Joined(SessionEventPayload {
        actor: player,
        session,
        reason
    });
    let serialized = json!(session_joined);
    return build_success_res(&serialized.to_string());
}

#[derive(Deserialize)]
struct SessionJoinDto {
    pub session_id: String
}
