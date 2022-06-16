use serde::{Deserialize, Serialize};
use pong::game_field::Input;
use crate::player::Player;
use crate::session::Session;

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionEventListDTO {
    pub session_id: String,
    pub events: Vec<SessionEventWriteDTO>,
}

#[derive(Serialize, Deserialize)]
pub enum PongEventDto<'a> {
    Move(&'a str, MoveEventPayload<'a>),
    Input(&'a str, InputEventPayload<'a>),
    Status(&'a str, StatusEventPayload),
    Session(&'a str, SessionEvent)
}

#[derive(Serialize, Deserialize)]
pub struct MoveEventPayload<'a> {
    pub id: i32,
    pub orientation_x: f64,
    pub orientation_y: f64,
    pub shape_param_1: f64,
    pub shape_param_2: f64,
    pub vel_x: f64,
    pub vel_y: f64,
    pub x: f64,
    pub y: f64,
    pub session_id: &'a str,
    pub ts: u128
}

#[derive(Serialize, Deserialize)]
pub struct InputEventPayload<'a> {
    pub inputs: Vec<Input>,
    pub player: &'a str,
    pub session_id: &'a str,
    pub ts: u128
}

#[derive(Serialize, Deserialize)]
pub struct StatusEventPayload {
    // TODO
}

#[derive(Serialize, Deserialize)]
pub enum SessionEvent {
    SessionCreated(SessionCreatedPayload),
    SessionJoined,
    SessionClosed
}

#[derive(Serialize, Deserialize)]
pub struct SessionCreatedPayload {
    pub session: Session,
    pub player: Player,
}

#[derive(Serialize, Deserialize)]
pub struct SessionJoinedPayload {
    pub session: Session,
    pub player: Player,
}

#[derive(Serialize, Deserialize)]
pub struct SessionClosedPayload {
    pub session: Session,
    pub reason: String,
}
