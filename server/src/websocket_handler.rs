use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use hyper_tungstenite::HyperWebsocket;
use tokio::sync::Mutex;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use hyper_tungstenite::tungstenite::{Error, Message};
use serde_json::json;
use tokio::time::sleep;
use serde::{Serialize, Deserialize};
use pong::game_field::Input;
use crate::event::{HeartBeatEventPayload, InputEventPayload, MoveEventPayload, SessionEvent, SessionEventListDTO, SessionEventPayload, SessionEventType};
use crate::actor::Player;
use crate::session::Session;
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
        let websocket = self.websocket.await?;
        let (mut websocket_writer, mut websocket_reader) = websocket.split();

        let session_manager = self.session_manager.lock().await;
        let event_handler_pair = session_manager.split(
            &self.websocket_session.session.hash,
            self.websocket_session.connection_type.get_topics(),
        );
        if let Err(_) = event_handler_pair {
            eprintln!(
                "Failed to create event reader/writer pair session: {:?}",
                self.websocket_session
            );
            return Err(Error::ConnectionClosed); // TODO: Use proper error for this case to close the connection
        }

        let (mut event_reader, mut event_writer) = event_handler_pair.unwrap();
        let websocket_session_read_copy = self.websocket_session.clone();
        tokio::spawn(async move {
            println!(
                "Ready to read messages from ws connection: {:?}",
                websocket_session_read_copy
            );
            while let Some(message) = websocket_reader.next().await {
                match message.unwrap() {
                    Message::Text(msg) => {
                        let ws_message = deserialize_ws_event(&msg, &websocket_session_read_copy.connection_type);
                        println!("Received ws message to persist events to kafka");
                        if let Err(e) = ws_message {
                            eprintln!("Failed to deserialize ws message to event {}: {}", msg, e);
                            continue;
                        }
                        let ws_message = ws_message.unwrap();
                        {
                            let session_id = ws_message.session_id();
                            if session_id != websocket_session_read_copy.session.hash {
                                eprintln!("Websocket has session {:?} but was asked to write to session {} - skip.", websocket_session_read_copy, session_id);
                                continue;
                            }
                        }
                        match ws_message {
                            WebsocketEvent::Snapshot(_, session_snapshot) => {
                                let mut any_error = false;
                                match session_snapshot {
                                    SessionSnapshot::Host(session_id, payload) => {
                                        let move_events = payload.objects.iter().map(|o| {
                                            o.to_move_event(&session_id, payload.ts)
                                        }).collect();
                                        any_error = write_events(move_events, "move", &mut event_writer) || any_error;
                                        let input_event = InputEventPayload {
                                            inputs: payload.inputs,
                                            player: websocket_session_read_copy.player.id.to_owned(),
                                            ts: payload.ts,
                                            session_id: session_id.to_owned()
                                        };
                                        any_error = write_events(vec![input_event], "input", &mut event_writer) || any_error;
                                        // TODO: Status events.
                                    },
                                    SessionSnapshot::Peer(session_id, payload) => {
                                        let input_event = InputEventPayload {
                                            inputs: payload.inputs,
                                            player: websocket_session_read_copy.player.id.to_owned(),
                                            ts: payload.ts,
                                            session_id: session_id.to_owned()
                                        };
                                        any_error = write_events(vec![input_event], "input", &mut event_writer) || any_error;
                                    },
                                    SessionSnapshot::Observer(_, _) => {
                                        // noop
                                    }
                                }
                                if any_error {
                                    eprintln!("At least one event write operation failed for session {:?}", websocket_session_read_copy);
                                }
                            },
                            WebsocketEvent::HeartBeat(session_id, heartbeat) => {
                                let event = HeartBeatEventPayload {
                                    session_id: session_id.clone(),
                                    actor_id: heartbeat.player,
                                    ts: heartbeat.ts
                                };
                                let res = write_events(vec![event], "heart_beat", &mut event_writer);
                                if !res {
                                    eprintln!("Failed to persist heart beat session {}", session_id);
                                }
                            }
                        }
                    }
                    Message::Close(msg) => {
                        // No need to send a reply: tungstenite takes care of this for you.
                        let reason = if let Some(msg) = &msg {
                            println!(
                                "Received close message with code {} and message: {}",
                                msg.code, msg.reason
                            );
                            format!("{}: {}", msg.code, msg.reason)
                        } else {
                            println!("Received close message");
                            "unknown".to_owned()
                        };

                        let session_closed_event = SessionEvent::Closed(SessionEventPayload {
                            actor: websocket_session_read_copy.player.clone(),
                            session: websocket_session_read_copy.session.clone(),
                            reason: format!("ws closed: {}", reason),
                        });
                        let msg = json!(session_closed_event).to_string();
                        let session_event_write_res = event_writer.write_to_session("session", vec![&msg]);
                        if let Err(e) = session_event_write_res {
                            eprintln!("Failed to write session closed event: {0}", e)
                        }
                        break;
                    }
                    _ => {}
                }
            }
            println!("!!!! Exit websocket receiver !!!!")
        });
        let websocket_session_write_copy = self.websocket_session.clone();
        tokio::spawn(async move {
            println!(
                "Ready to read messages from kafka: {:?}",
                websocket_session_write_copy
            );
            loop {
                println!("Reading messages from kafka.");
                let messages = event_reader.read_from_session();
                if let Err(_) = messages {
                    eprintln!(
                        "Failed to read messages from kafka for session: {:?}",
                        websocket_session_write_copy
                    );
                    continue;
                }
                // println!("Read messages for websocket_session {:?} from consumer: {:?}", websocket_session_write_copy, messages);
                let messages = messages.unwrap();
                if messages.len() == 0 {
                    println!("No new messages from kafka.");
                } else {
                    println!("{} new messages from kafka.", messages.len());
                    let json = serde_json::to_string(&messages).unwrap();
                    let message = Message::from(json);
                    println!("Sending kafka messages through websocket.");
                    let send_res = websocket_writer.send(message).await;
                    if let Err(e) = send_res {
                        eprintln!(
                            "Failed to send message to websocket for session {:?}: {:?}",
                            websocket_session_write_copy, e
                        );
                        break;
                    }
                }
                // Avoid starvation of read thread (?)
                // TODO: How to avoid this? This is very bad for performance.
                sleep(Duration::from_millis(1)).await;
            }
        });
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WebSocketSession {
    pub connection_type: WebSocketConnectionType,
    pub session: Session,
    pub player: Player
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
            WebSocketConnectionType::HOST => &["input", "session"],
            WebSocketConnectionType::PEER | WebSocketConnectionType::OBSERVER => {
                &["move", "input", "status", "session"]
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
    pub player: String,
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

#[derive(Deserialize)]
struct HostSessionSnapshotDTO {
    pub session_id: String,
    pub inputs: Vec<Input>,
    pub objects: Vec<GameObjectStateDTO>,
    pub player: String,
    pub ts: u128
}

#[derive(Deserialize)]
struct PeerSessionSnapshotDTO {
    pub session_id: String,
    pub inputs: Vec<Input>,
    pub player: String,
    pub ts: u128
}

#[derive(Deserialize)]
struct ObserverSessionSnapshotDTO {
    pub session_id: String,
    pub player: String,
    pub ts: u128
}

#[derive(Deserialize)]
struct GameObjectStateDTO {
    pub id: i32,
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
            id: self.id,
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

fn write_events<T>(events: Vec<T>, topic: &str, event_writer: &mut SessionWriter) -> bool where T : Serialize + Debug {
    let mut any_error = false;
    let mut to_send = vec![];
    for event in events {
        let serialized = serde_json::to_string(&event);
        if let Err(e) = serialized {
            eprintln!("Failed to serialize event {:?} in topic {}: {:?}", event, topic, e);
            any_error = true;
            continue;
        }
        let serialized = serialized.unwrap();
        to_send.push(serialized);
    }

    let to_send = to_send.iter().map(|e| e.as_str()).collect();
    let write_res = event_writer.write_to_session(topic, to_send);
    if let Err(e) = write_res {
        eprintln!("Failed to write at least one event to topic {}: {:?}", topic, e);
        any_error = true;
    }
    return any_error;
}
