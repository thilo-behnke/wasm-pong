use std::str::FromStr;
use serde::{Deserialize, Serialize};
use pong::game_field::{GameScore, Input};
use crate::actor::{Actor, Player};
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
pub struct MoveEventBatchPayload {
    pub session_id: String,
    pub ts: u128,
    pub objects: Vec<MoveEventPayload>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoveEventPayload {
    pub session_id: String,
    pub id: String,
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

#[derive(Debug, Serialize)]
pub struct TickEvent {
    pub tick: u128,
    pub objects: Vec<MoveEventPayload>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputEventPayload {
    pub session_id: String,
    pub inputs: Vec<Input>,
    pub player_id: String,
    pub ts: u128,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusEventPayload {
    pub session_id: String,
    pub score: GameScore,
    pub winner: Option<String>,
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
    ObserverAdded(SessionEventPayload),
    Closed(SessionEventPayload),
}

impl SessionEvent {
    pub fn session_id(&self) -> &str {
        return match self {
            SessionEvent::Created(e) | SessionEvent::Joined(e) | SessionEvent::ObserverAdded(e) | SessionEvent::Closed(e) => e.session_id()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SessionEventPayload {
    pub session: Session,
    pub actor: Actor,
    pub reason: String,
}

impl SessionEventPayload {
    pub fn session_id(&self) -> &str {
        return &self.session.session_id;
    }
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

#[cfg(test)]
mod tests {
    use crate::event::{SessionEvent, SessionEventPayload};
    use crate::actor::{Actor, Observer, Player};
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
                session_id: "abc".to_owned(),
                state: SessionState::PENDING,
                players: vec![Player { id: "player_1".to_owned(), nr: 1, ip: "127.0.0.1".to_owned() }],
                observers: vec![Observer {id: "observer_1".to_owned(), ip: "127.0.0.1".to_owned()}]
            },
            actor: Actor::Player(Player { id: "player_1".to_owned(), nr: 1, ip: "127.0.0.1".to_owned() }),
            reason: "some reason".to_owned(),
        })
    }
}
