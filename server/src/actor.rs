use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Observer {
    pub id: String,
}
