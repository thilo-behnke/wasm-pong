use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use futures::{stream::StreamExt};
use futures::future::err;
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper_tungstenite::HyperWebsocket;
use hyper_tungstenite::tungstenite::{Error};
use log::{debug, error, info};
use tokio::sync::Mutex;

use crate::request_handler::{DefaultRequestHandler, RequestHandler};
use crate::session_manager::{SessionManager};
use crate::utils::http_utils::{build_error_res, get_query_params, read_json_body};
use crate::websocket_handler::{DefaultWebsocketHandler, WebSocketConnectionType, WebsocketHandler, WebSocketSession};

pub struct HttpServer {
    addr: [u8; 4],
    port: u16,
    session_manager: Arc<Mutex<SessionManager>>,
}

impl HttpServer {
    pub fn new(
        addr: [u8; 4],
        port: u16,
        kafka_host: &str,
        kafka_topic_manager_host: &str,
    ) -> HttpServer {
        let session_manager = Arc::new(Mutex::new(SessionManager::new(
            kafka_host,
            kafka_topic_manager_host,
        )));
        HttpServer {
            addr,
            port,
            session_manager,
        }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let make_svc = make_service_fn(|socket: &AddrStream| {
            let session_manager = Arc::clone(&self.session_manager);
            let addr = socket.remote_addr();
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let session_manager = Arc::clone(&session_manager);
                    async move {
                        if hyper_tungstenite::is_upgrade_request(&req) {
                            return handle_potential_ws_upgrade(session_manager, req, addr).await;
                        }

                        return handle_http_request(session_manager, req, addr).await;
                    }
                }))
            }
        });

        let host = (self.addr, self.port).into();
        let server = Server::bind(&host).serve(make_svc);
        info!("running server on http://{}", host);
        let graceful = server.with_graceful_shutdown(shutdown_signal());
        graceful.await?;
        Ok(())
    }

}

async fn handle_potential_ws_upgrade(session_manager: Arc<Mutex<SessionManager>>, req: Request<Body>, addr: SocketAddr) -> Result<Response<Body>, Infallible> {
    debug!(
        "received request from {:?} to upgrade to websocket connection: {:?}",
        addr, req
    );
    let params = get_query_params(&req);
    debug!("ws request params: {:?}", params);
    if !params.contains_key("session_id") {
        error!("Missing session id request param for websocket connection, don't upgrade connection to ws.");
        return build_error_res(
            "Missing request param: session_id",
            StatusCode::BAD_REQUEST,
        );
    }
    if !params.contains_key("player_id") {
        error!("Missing player id request param for websocket connection, don't upgrade connection to ws.");
        return build_error_res(
            "Missing request param: player_id",
            StatusCode::BAD_REQUEST,
        );
    }
    if !params.contains_key("connection_type") {
        error!("Missing connection type request param for websocket connection, don't upgrade connection to ws.");
        let res = build_error_res(
            "Missing request param: connection_type",
            StatusCode::BAD_REQUEST,
        );
        return res;
    }
    let request_session_id = *params.get("session_id").unwrap();
    let request_player_id = *params.get("player_id").unwrap();
    let request_player_ip = addr.ip().to_string();
    let session = session_manager.lock().await.get_session(request_session_id);
    if let None = session {
        let error = format!("Session does not exist: {}", request_session_id);
        error!("{}", error);
        return build_error_res(error.as_str(), StatusCode::NOT_FOUND);
    }
    let session = session.unwrap();
    let matching_player = session.players.iter().find(|p| p.id == request_player_id);
    if let None = matching_player {
        let error = format!("Player is not registered in session: {}", request_player_id);
        error!("{}", error);
        return build_error_res(error.as_str(), StatusCode::FORBIDDEN);
    }
    let matching_player = matching_player.unwrap();
    if matching_player.ip != request_player_ip {
        let error = format!("Player with wrong ip tried to join session: {} (expected) vs {} (actual)", matching_player.ip, request_player_ip);
        error!("{}", error);
        return build_error_res(error.as_str(), StatusCode::FORBIDDEN);
    }
    let connection_type_raw = params.get("connection_type").unwrap();
    let connection_type =
        WebSocketConnectionType::from_str(connection_type_raw);
    if let Err(_) = connection_type {
        let error =
            format!("Invalid connection type: {}", connection_type_raw);
        error!("{}", error);
        return build_error_res(error.as_str(), StatusCode::BAD_REQUEST);
    }
    let websocket_session = WebSocketSession {
        session: session.clone(),
        connection_type: connection_type.unwrap(),
        player: matching_player.clone()
    };
    debug!("websocket upgrade request is valid, will now upgrade to websocket: {:?}", req);

    let (response, websocket) =
        hyper_tungstenite::upgrade(req, None).unwrap();

    // Spawn a task to handle the websocket connection.
    tokio::spawn(async move {
        if let Err(e) =
        serve_websocket(websocket_session, websocket, session_manager)
            .await
        {
            error!("Error in websocket connection: {:?}", e);
        }
    });

    debug!("websocket upgrade done.");
    // Return the response so the spawned future can continue.
    return Ok(response);
}

/// Handle a websocket connection.
async fn serve_websocket(
    websocket_session: WebSocketSession,
    websocket: HyperWebsocket,
    session_manager: Arc<Mutex<SessionManager>>,
) -> Result<(), Error> {
    let handler = DefaultWebsocketHandler::new(
        websocket_session, websocket, session_manager,
    );
    handler.serve().await
}

async fn handle_http_request(
    session_manager: Arc<Mutex<SessionManager>>,
    req: Request<Body>,
    addr: SocketAddr,
) -> Result<Response<Body>, Infallible> {
    let handler = DefaultRequestHandler::new(
        session_manager
    );
    handler.handle(req, addr).await
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    let shutdown_received = tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
    info!("received shutdown signal, shutting down now...");
}
