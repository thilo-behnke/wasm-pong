use serde::{Serialize, Deserialize};
use crate::player::Player;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: u16,
    pub hash: String,
    pub state: SessionState,
    pub players: Vec<Player>,
}

impl Session {
    pub fn new(id: u16, hash: String, player: Player) -> Session {
        Session {
            players: vec![player],
            id,
            hash,
            state: SessionState::PENDING,
        }
    }

    pub fn can_be_joined(&self) -> bool {
        self.players.len() == 1
    }

    pub fn join(&mut self, player: Player) -> bool {
        if !self.can_be_joined() {
            return false;
        }
        self.players.push(player);
        return true;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum SessionState {
    PENDING, // 1 player is missing
    RUNNING, // game is playing
    CLOSED,  // game is over
}

