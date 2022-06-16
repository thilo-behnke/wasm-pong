use std::str::FromStr;
use serde::{Deserialize, Serialize};
use pong::game_field::Input;
use crate::player::Player;
use crate::session::Session;

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionEventListDTO<'a> {
    pub session_id: &'a str,
    pub events: Vec<PongEvent<'a>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PongEventWrapper<'a> {
    pub session_id: &'a str,
    pub topic: &'a str,
    pub event: &'a str
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PongEvent<'a> {
    Move(&'a str, MoveEventPayload<'a>),
    Input(&'a str, InputEventPayload<'a>),
    Status(&'a str, StatusEventPayload),
    Session(&'a str, SessionEvent)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoveEventPayload<'a> {
    pub session_id: &'a str,
    pub id: i32,
    pub orientation_x: f64,
    pub orientation_y: f64,
    pub shape_param_1: f64,
    pub shape_param_2: f64,
    pub vel_x: f64,
    pub vel_y: f64,
    pub x: f64,
    pub y: f64,
    pub ts: u128
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputEventPayload<'a> {
    pub session_id: &'a str,
    pub inputs: Vec<Input>,
    pub player: &'a str,
    pub ts: u128
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusEventPayload {
    // TODO
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SessionEvent {
    SessionCreated(SessionEventPayload),
    SessionJoined(SessionEventPayload),
    SessionClosed(SessionEventPayload)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionEventPayload {
    pub session: Session,
    pub player: Player,
    pub event_type: SessionEventType
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SessionEventType {
    Created, Joined, Closed
}

impl FromStr for SessionEventType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "created" => Ok(SessionEventType::Created),
            "joined" => Ok(SessionEventType::Joined),
            "closed" => Ok(SessionEventType::Closed),
            _ => Err(())
        }
    }
}

pub fn deserialize(event: &str) -> Option<PongEvent> {
    let wrapper = serde_json::from_str::<PongEventWrapper>(event);
    wrapper.ok().and_then(|w| {
        match w.topic {
            "move" => serde_json::from_str::<MoveEventPayload>(w.event).ok().map(|e| PongEvent::Move(w.session_id, e)),
            "input" => serde_json::from_str::<InputEventPayload>(w.event).ok().map(|e| PongEvent::Input(w.session_id, e)),
            "status" => serde_json::from_str::<StatusEventPayload>(w.event).ok().map(|e| PongEvent::Status(w.session_id, e)),
            "session" => serde_json::from_str::<SessionEventPayload>(w.event).ok().map(|e| deserialize_session_event(w.session_id, e)),
            _ => None
        }
    })
}

pub fn deserialize_session_event(session_id: &str, event: SessionEventPayload) -> PongEvent {
    let session_event = match event.event_type {
        SessionEventType::Created => SessionEvent::SessionCreated(event),
        SessionEventType::Joined => SessionEvent::SessionJoined(event),
        SessionEventType::Closed => SessionEvent::SessionClosed(event)
    };

    PongEvent::Session(session_id, session_event)
}
