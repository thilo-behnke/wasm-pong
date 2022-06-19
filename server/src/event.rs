use std::str::FromStr;
use serde::{Deserialize, Serialize};
use pong::game_field::Input;
use crate::actor::Player;
use crate::session::Session;

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionEventListDTO {
    pub session_id: String,
    pub events: Vec<PongEventWrapper>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PongEventWrapper {
    pub session_id: String,
    pub topic: String,
    pub event: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PongEvent {
    Move(String, MoveEventPayload),
    Input(String, InputEventPayload),
    Status(String, StatusEventPayload),
    HeartBeat(String, HeartBeatEventPayload),
    Session(String, SessionEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoveEventPayload {
    pub session_id: String,
    pub id: i32,
    pub orientation_x: f64,
    pub orientation_y: f64,
    pub shape_param_1: f64,
    pub shape_param_2: f64,
    pub vel_x: f64,
    pub vel_y: f64,
    pub x: f64,
    pub y: f64,
    pub ts: u128,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputEventPayload {
    pub session_id: String,
    pub inputs: Vec<Input>,
    pub player: String,
    pub ts: u128,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusEventPayload {
    // TODO
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartBeatEventPayload {
    pub actor_id: String,
    pub session_id: String,
    pub ts: u128
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type")]
pub enum SessionEvent {
    Created(SessionEventPayload),
    Joined(SessionEventPayload),
    Closed(SessionEventPayload),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SessionEventPayload {
    pub session: Session,
    pub actor: Player,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SessionEventType {
    Created,
    Joined,
    Closed,
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
        match w.topic.as_str() {
            "move" => serde_json::from_str::<MoveEventPayload>(&w.event).ok().map(|e| PongEvent::Move(w.session_id, e)),
            "input" => serde_json::from_str::<InputEventPayload>(&w.event).ok().map(|e| PongEvent::Input(w.session_id, e)),
            "status" => serde_json::from_str::<StatusEventPayload>(&w.event).ok().map(|e| PongEvent::Status(w.session_id, e)),
            "heart_beat" => serde_json::from_str::<HeartBeatEventPayload>(&w.event).ok().map(|e| PongEvent::HeartBeat(w.session_id, e)),
            "session" => serde_json::from_str::<SessionEvent>(&w.event).ok().map(|e| PongEvent::Session(w.session_id, e)),
            _ => None
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::event::{SessionEvent, SessionEventPayload};
    use crate::actor::{Observer, Player};
    use crate::session::{Session, SessionState};

    const SESSION_EVENT_JSON: &str = "{\"event_type\":\"Created\",\"session\":{\"id\":1,\"hash\":\"abc\",\"state\":\"PENDING\",\"players\":[{\"id\":\"player_1\"}],\"observers\":[{\"id\":\"observer_1\"}]},\"actor\":{\"id\":\"player_1\"},\"reason\":\"some reason\"}";

    #[test]
    pub fn should_serialize_correctly() {
        let res = serde_json::to_string(&get_session_event());
        assert_eq!(res.is_ok(), true);
        let res = res.unwrap();
        assert_eq!(res, SESSION_EVENT_JSON);
    }

    #[test]
    pub fn should_deserialize_correctly() {
        let res = serde_json::from_str::<SessionEvent>(&SESSION_EVENT_JSON);
        assert_eq!(res.is_ok(), true);
        let res = res.unwrap();
        assert_eq!(res, get_session_event());
    }

    fn get_session_event() -> SessionEvent {
        SessionEvent::Created(SessionEventPayload {
            session: Session {
                id: 1,
                hash: "abc".to_owned(),
                state: SessionState::PENDING,
                players: vec![Player { id: "player_1".to_owned() }],
                observers: vec![Observer {id: "observer_1".to_owned()}]
            },
            actor: Player { id: "player_1".to_owned() },
            reason: "some reason".to_owned(),
        })
    }
}
