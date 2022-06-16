use serde::{Deserialize, Serialize};
use crate::player::Player;
use crate::session::Session;

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionEventListDTO {
    pub session_id: String,
    pub events: Vec<SessionEventWriteDTO>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionEventWriteDTO {
    pub session_id: String,
    pub topic: String,
    pub msg: String,
}

#[derive(Debug, Serialize)]
pub struct SessionClosedDto {
    pub session: Session,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionReadDTO {
    pub session_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionJoinDto {
    pub session_id: String,
}

#[derive(Debug, Serialize)]
pub struct SessionJoinedDto {
    pub session: Session,
    pub player: Player,
}

#[derive(Debug, Serialize)]
pub struct SessionCreatedDto {
    pub session: Session,
    pub player: Player,
}

