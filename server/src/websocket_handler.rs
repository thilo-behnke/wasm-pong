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
use crate::event::{SessionEventListDTO};
use crate::session::Session;
use crate::session_manager::{SessionManager};

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
                            let serialized = serde_json::to_string(&event);
                            if let Err(e) = serialized {
                                eprintln!("Failed to serialize event {:?}: {}", event, e);
                                any_error = true;
                                continue;
                            }
                            let write_res = event_writer.write_to_session(&event.topic, &serialized.unwrap());
                            if let Err(e) = write_res {
                                any_error = true;
                                eprintln!("Failed to write event {:?}: {}", event, e);
                            }
                        }
                        if any_error {
                            eprintln!(
                                "Failed to write at least one message for session {}",
                                event_wrapper.session_id
                            );
                        } else {
                            println!(
                                "Successfully wrote {} messages to kafka for session {:?}",
                                event_count, websocket_session_read_copy
                            )
                        }
                    }
                    Message::Close(msg) => {
                        // No need to send a reply: tungstenite takes care of this for you.
                        if let Some(msg) = &msg {
                            println!(
                                "Received close message with code {} and message: {}",
                                msg.code, msg.reason
                            );
                        } else {
                            println!("Received close message");
                        }

                        let session_closed_event = SessionClosedDto {
                            session: websocket_session_read_copy.session.clone(),
                            reason: "ws closed".to_owned(),
                        };
                        let msg = json!(session_closed_event).to_string();
                        let session_event_write_res = event_writer.write_to_session("session", &msg);
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
