use std::fmt::{Debug};
use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use hyper_tungstenite::HyperWebsocket;
use hyper_tungstenite::tungstenite::{Error, Message};
use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Mutex;
use tokio::task;

use pong::event::event::{EventWriter};
use pong::game_field::{GameState, Input};

use crate::actor::{Actor};
use crate::event::{HeartBeatEventPayload, MoveEventBatchPayload, MoveEventPayload, SessionEvent, SessionEventListDTO, SessionEventPayload, SessionEventType, StatusEventPayload, TickEvent};
use crate::session::{Session, SessionState};
use crate::session_manager::{SessionManager, SessionWriter};

#[async_trait]
pub trait WebsocketHandler {
    async fn serve(self) -> Result<(), Error> ;
}

pub struct DefaultWebsocketHandler {
    websocket_session: WebSocketSession,
    websocket: HyperWebsocket,
    session_manager: Arc<Mutex<SessionManager>>,
}

impl DefaultWebsocketHandler {
    pub fn new(
        websocket_session: WebSocketSession,
        websocket: HyperWebsocket,
        session_manager: Arc<Mutex<SessionManager>>,
    ) -> DefaultWebsocketHandler {
        DefaultWebsocketHandler {
            websocket_session, websocket, session_manager
        }
    }
}



#[async_trait]
impl WebsocketHandler for DefaultWebsocketHandler {
    async fn serve(self) -> Result<(), Error> {
        info(&self.websocket_session, "serving new websocket connection");
        let websocket = self.websocket.await?;
        let (mut websocket_writer, mut websocket_reader) = websocket.split();

        let event_handler_pair = async {
            let session_manager = self.session_manager.lock().await;
            session_manager.split(
                &self.websocket_session.session.session_id,
                self.websocket_session.connection_type.get_topics(),
            ).await
        }.await;
        if let Err(_) = event_handler_pair {
            error(
                &self.websocket_session,
                &format!(
                    "failed to create event reader/writer pair session: {:?}",
                    self.websocket_session
                )
            );
            return Err(Error::ConnectionClosed); // TODO: Use proper error for this case to close the connection
        }

        let (mut event_reader, mut event_writer) = event_handler_pair.unwrap();
        let websocket_session_read_copy = self.websocket_session.clone();
        tokio::spawn(async move {
            info(
                &websocket_session_read_copy,
                "ready to read messages from ws connection",
            );
            while let Some(message) = websocket_reader.next().await {
                if let Err(e) = message {
                    error(&websocket_session_read_copy, &format!("ws message read failed for session: {:?}", e));
                    let reason = format!("ws closed: {:?}", e);
                    write_session_close_event(&mut event_writer, &websocket_session_read_copy, reason.as_str()).await;
                    break;
                }
                let message = message.unwrap();
                trace(&websocket_session_read_copy, &format!("read new message from websocket: {:?}", message));
                match message {
                    Message::Text(msg) => {
                        let ws_message = deserialize_ws_event(&msg, &websocket_session_read_copy.connection_type);
                        trace(&websocket_session_read_copy, "received ws event to persist to kafka");
                        if let Err(e) = ws_message {
                            error(&websocket_session_read_copy, &format!("Failed to deserialize ws message to event: {:?}", e));
                            continue;
                        }
                        let ws_message = ws_message.unwrap();
                        {
                            let session_id = ws_message.session_id();
                            if session_id != websocket_session_read_copy.session.session_id {
                                error(&websocket_session_read_copy, &format!("websocket was asked to write to other session {} - skip.", session_id));
                                continue;
                            }
                        }
                        match ws_message {
                            WebsocketEvent::Snapshot(_, session_snapshot) => {
                                trace!("received message is snapshot");
                                let mut any_error = false;
                                match session_snapshot {
                                    SessionSnapshot::Host(_, payload) => {
                                        trace(&websocket_session_read_copy, "received message is HOST snapshot");
                                        let write_res = write_events(vec![payload], "host_tick", &mut event_writer).await;
                                        if !write_res {
                                            error(&websocket_session_read_copy, "failed to write HOST tick");
                                        }
                                        any_error = !write_res;
                                    },
                                    SessionSnapshot::Peer(session_id, payload) => {
                                        trace(&websocket_session_read_copy, "received message is PEER snapshot");
                                        let write_res = write_events(vec![payload], "peer_tick", &mut event_writer).await;
                                        if !write_res {
                                            error(&websocket_session_read_copy, &format!("failed to write PEER tick"));
                                        }
                                        any_error = !write_res;
                                    },
                                    SessionSnapshot::Observer(_, _) => {
                                        // noop
                                    }
                                }
                                if any_error {
                                    error(&websocket_session_read_copy, "at least one event write operation failed");
                                } else {
                                    debug(&websocket_session_read_copy, "successfully persisted session snapshot");
                                }
                            },
                            WebsocketEvent::HeartBeat(session_id, heartbeat) => {
                                trace(&websocket_session_read_copy, "received message is heartbeat");
                                let event = HeartBeatEventPayload {
                                    session_id: session_id.clone(),
                                    actor_id: heartbeat.player_id,
                                    ts: heartbeat.ts
                                };
                                let res = write_events(vec![event], "heart_beat", &mut event_writer).await;
                                if !res {
                                    error!("failed to persist heart beat.");
                                } else {
                                    debug(&websocket_session_read_copy, "successfully persisted heartbeat");
                                }
                            }
                        }
                    }
                    Message::Close(msg) => {
                        info(&websocket_session_read_copy, "ws session closed");
                        // No need to send a reply: tungstenite takes care of this for you.
                        let reason = if let Some(msg) = &msg {
                            debug!(
                                "Received close message with code {} and message: {}",
                                msg.code, msg.reason
                            );
                            format!("{}: {}", msg.code, msg.reason)
                        } else {
                            "unknown".to_owned()
                        };

                        let reason = format!("ws closed: {}", reason);
                        write_session_close_event(&mut event_writer, &websocket_session_read_copy, reason.as_str()).await;
                        break;
                    }
                    _ => {}
                }

                trace(&websocket_session_read_copy, "kafka write done, waiting for next cycle.");
                task::yield_now().await;
            }
            info!("ws receiver terminated")
        });
        let websocket_session_write_copy = self.websocket_session.clone();
        tokio::spawn(async move {
            debug(
                &websocket_session_write_copy,
                "ready to read messages from kafka"
            );
            loop {
                trace(&websocket_session_write_copy, "reading messages from kafka");
                // TODO: Should perform more filtering, e.g. inputs of player are not relevant.
                let events = event_reader.read_from_session().await;
                if let Err(e) = events {
                    error(&websocket_session_write_copy, &format!("Failed to read messages from kafka: {:?}", e));
                    continue;
                }
                let events = events.unwrap();
                trace(&websocket_session_write_copy, &format!("read messages for websocket_session from consumer: {:?}", events));

                if events.len() == 0 {
                    trace(&websocket_session_write_copy, "no new messages from kafka.");
                } else {
                    let mut session_events = events.iter().filter(|e| e.topic == "session")
                        .map(|e| WebsocketEventDTO {
                            topic: "session".to_owned(),
                            event: e.event.clone()
                        })
                        .collect();

                    let mut tick_events = match websocket_session_write_copy.connection_type {
                        WebSocketConnectionType::HOST => {
                            events.iter().filter(|e| e.topic == "peer_tick")
                                .map(|e| WebsocketEventDTO {topic: "tick".to_owned(), event: e.event.clone()})
                                .collect()
                        },
                        _ => {
                            events.iter().filter(|e| e.topic == "host_tick")
                                .map(|e| WebsocketEventDTO {topic: "tick".to_owned(), event: e.event.clone()})
                                .collect()
                        }
                    };

                    let mut event_dtos = vec![];
                    event_dtos.append(&mut session_events);
                    event_dtos.append(&mut tick_events);

                    trace(&websocket_session_write_copy, &format!("{} new messages from kafka.", event_dtos.len()));
                    let json = serde_json::to_string(&event_dtos).unwrap();
                    trace(&websocket_session_write_copy, &format!("sending msg batch to client: {}", json));
                    let message = Message::from(json);
                    trace(&websocket_session_write_copy, "sending kafka messages through websocket.");
                    let send_res = websocket_writer.send(message).await;
                    if let Err(e) = send_res {
                        error(
                            &websocket_session_write_copy,
                            &format!(
                                "Failed to send message to websocket: {:?}", e
                            )
                        );
                        break;
                    }
                }

                trace(&websocket_session_write_copy, "kafka read done, waiting for next cycle.");
                task::yield_now().await;
            }
        });
        Ok(())
    }
}


// TODO: doable in macro?
fn trace(websocket_session: &WebSocketSession, msg: &str) {
    trace!("[{}] {}", websocket_session.session.session_id, msg)
}

fn debug(websocket_session: &WebSocketSession, msg: &str) {
    debug!("[{}] {}", websocket_session.session.session_id, msg)
}

fn info(websocket_session: &WebSocketSession, msg: &str) {
    info!("[{}] {}", websocket_session.session.session_id, msg)
}

fn error(websocket_session: &WebSocketSession, msg: &str) {
    error!("[{}] {}", websocket_session.session.session_id, msg)
}

async fn write_session_close_event(event_writer: &mut SessionWriter, websocket_session: &WebSocketSession, close_reason: &str) {
    let mut updated_session = websocket_session.session.clone();
    updated_session.state = SessionState::CLOSED;
    let session_closed_event = SessionEvent::Closed(SessionEventPayload {
        actor: websocket_session.actor.clone(),
        session: updated_session,
        reason: format!("ws closed: {}", close_reason),
    });
    let msg = json!(session_closed_event).to_string();
    let session_event_write_res = event_writer.write_to_session("session", vec![&msg]).await;
    if let Err(e) = session_event_write_res {
        eprintln!("Failed to write session closed event: {0}", e)
    }
}

#[derive(Debug, Clone)]
pub struct WebSocketSession {
    pub connection_type: WebSocketConnectionType,
    pub session: Session,
    pub actor: Actor
}

#[derive(Debug, Clone, PartialEq)]
pub enum WebSocketConnectionType {
    HOST,
    PEER,
    OBSERVER,
}

impl FromStr for WebSocketConnectionType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "host" => Ok(WebSocketConnectionType::HOST),
            "peer" => Ok(WebSocketConnectionType::PEER),
            "observer" => Ok(WebSocketConnectionType::OBSERVER),
            _ => Err(()),
        }
    }
}

impl WebSocketConnectionType {
    pub fn get_topics(&self) -> &[&str] {
        match self {
            WebSocketConnectionType::HOST => &["peer_tick", "session"],
            WebSocketConnectionType::PEER | WebSocketConnectionType::OBSERVER => {
                &["host_tick", "session"]
            }
        }
    }
}

fn deserialize_ws_event(message: &str, connection_type: &WebSocketConnectionType) -> Result<WebsocketEvent, String> {
    let deserialized = serde_json::from_str::<WebsocketMessageWrapper>(message);
    if let Err(e) = deserialized {
        let err = format!("Failed to deserialize ws message {}: {:?}", message, e);
        eprintln!("{}", err);
        return Err(err);
    }
    let deserialized = deserialized.unwrap();
    match deserialized.msg_type {
        WebsocketEventType::SessionSnapshot => {
            deserialize_event_snapshot(&deserialized.payload, connection_type).map(|s| WebsocketEvent::Snapshot(s.session_id().to_owned(), s))
        },
        WebsocketEventType::HeartBeat => {
            serde_json::from_str::<HeartBeatMessage>(&deserialized.payload).map_err(|e| e.to_string()).map(|h| WebsocketEvent::HeartBeat(h.session_id.clone(), h))
        }
    }
}

fn deserialize_event_snapshot(serialized_snapshot: &str, connection_type: &WebSocketConnectionType) -> Result<SessionSnapshot, String> {
    match connection_type {
        WebSocketConnectionType::HOST => serde_json::from_str::<HostSessionSnapshotDTO>(serialized_snapshot).map_err(|e| e.to_string()).map(|s| SessionSnapshot::Host(s.session_id.to_owned(), s)),
        WebSocketConnectionType::PEER => serde_json::from_str::<PeerSessionSnapshotDTO>(serialized_snapshot).map_err(|e| e.to_string()).map(|s| SessionSnapshot::Peer(s.session_id.to_owned(), s)),
        WebSocketConnectionType::OBSERVER => serde_json::from_str::<ObserverSessionSnapshotDTO>(serialized_snapshot).map_err(|e| e.to_string()).map(|s| SessionSnapshot::Observer(s.session_id.to_owned(), s)),
    }
}

enum WebsocketEvent {
    Snapshot(String, SessionSnapshot),
    HeartBeat(String, HeartBeatMessage)
}

impl WebsocketEvent {
    pub fn session_id(&self) -> &str {
        match self {
            WebsocketEvent::HeartBeat(s, _) => &s,
            WebsocketEvent::Snapshot(s, _) => &s,
        }
    }
}

#[derive(Deserialize)]
struct WebsocketMessageWrapper {
    pub msg_type: WebsocketEventType,
    pub payload: String
}

#[derive(Deserialize)]
enum WebsocketEventType {
    HeartBeat, SessionSnapshot
}

#[derive(Deserialize)]
struct HeartBeatMessage {
    pub player_id: String,
    pub session_id: String,
    pub ts: u128
}

enum SessionSnapshot {
    Host(String, HostSessionSnapshotDTO),
    Peer(String, PeerSessionSnapshotDTO),
    Observer(String, ObserverSessionSnapshotDTO)
}

impl SessionSnapshot {
    pub fn session_id(&self) -> &str {
        match self {
            SessionSnapshot::Host(id, _) => id,
            SessionSnapshot::Peer(id, _) => id,
            SessionSnapshot::Observer(id, _) => id
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct HostSessionSnapshotDTO {
    pub session_id: String,
    pub inputs: Vec<Input>,
    pub state: GameState,
    pub objects: Vec<GameObjectStateDTO>,
    pub player_id: String,
    pub ts: u128
}

#[derive(Deserialize, Serialize, Debug)]
struct PeerSessionSnapshotDTO {
    pub session_id: String,
    pub inputs: Vec<Input>,
    pub player_id: String,
    pub ts: u128
}

#[derive(Deserialize)]
struct ObserverSessionSnapshotDTO {
    pub session_id: String,
    pub player: String,
    pub ts: u128
}

#[derive(Deserialize, Serialize, Debug)]
struct GameObjectStateDTO {
    pub id: String,
    pub orientation_x: f64,
    pub orientation_y: f64,
    pub shape_param_1: f64,
    pub shape_param_2: f64,
    pub vel_x: f64,
    pub vel_y: f64,
    pub x: f64,
    pub y: f64,
}

impl GameObjectStateDTO {
    pub fn to_move_event(&self, session_id: &str, ts: u128) -> MoveEventPayload {
        MoveEventPayload {
            session_id: session_id.to_owned(),
            ts,
            id: self.id.clone(),
            x: self.x,
            y: self.y,
            orientation_x: self.orientation_x,
            orientation_y: self.orientation_y,
            vel_x: self.vel_x,
            vel_y: self.vel_y,
            shape_param_1: self.shape_param_1,
            shape_param_2: self.shape_param_2,
        }
    }
}

async fn write_events<T>(events: Vec<T>, topic: &str, event_writer: &mut SessionWriter) -> bool where T : Serialize + Debug {
    if events.len() == 0 {
        debug!("called to write 0 events - noop");
        return true;
    }
    let mut any_error = false;
    let mut to_send = vec![];
    for event in events {
        let serialized = serde_json::to_string(&event);
        if let Err(e) = serialized {
            error!("Failed to serialize event {:?} in topic {}: {:?}", event, topic, e);
            any_error = true;
            continue;
        }
        let serialized = serialized.unwrap();
        to_send.push(serialized);
    }

    let to_send = to_send.iter().map(|e| e.as_str()).collect();
    let write_res = event_writer.write_to_session(topic, to_send).await;
    if let Err(e) = write_res {
        error!("Failed to write at least one event to topic {}: {:?}", topic, e);
        any_error = true;
    }
    return !any_error;
}

#[derive(Debug, Serialize, Deserialize)]
struct WebsocketEventDTO {
    pub topic: String,
    pub event: String
}
