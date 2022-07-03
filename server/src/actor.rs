use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "actor_type")]
pub enum Actor {
    Player(Player),
    Observer(Observer)
}

impl Actor {
    pub fn id(&self) -> &str {
        match self {
            Actor::Player(p) => &p.id,
            Actor::Observer(o) => &o.id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub id: String,
    pub ip: String,
    pub nr: u8
}

impl Player {
    pub fn new(nr: u8, ip: String) -> Player {
        Player {
            ip,
            id: Uuid::new_v4().to_string(),
            nr
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Observer {
    pub id: String,
    pub ip: String,
}

impl Observer {
    pub fn new(ip: String) -> Observer {
        Observer {
            ip,
            id: Uuid::new_v4().to_string()
        }
    }
}
