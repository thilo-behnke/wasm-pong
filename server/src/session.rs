use crate::hash::Hasher;
use crate::kafka::KafkaTopicManager;
use serde::{Serialize};
use pong::event::event::{Event, EventReader, EventWriter};

#[derive(Debug)]
pub struct SessionManager {
    sessions: Vec<Session>,
    topic_manager: KafkaTopicManager
}

impl SessionManager {
    pub fn new() -> SessionManager {
        SessionManager {
            sessions: vec![],
            topic_manager: KafkaTopicManager::from("localhost:7243")
        }
    }

    pub async fn create_session(&mut self) -> Result<Session, String> {
        let add_partition_res = self.topic_manager.add_partition().await;
        if let Err(e) = add_partition_res {
            println!("Failed to create partition: {}", e);
            return Err(e);
        }
        let session_id = add_partition_res.unwrap();
        let session_hash = Hasher::hash(session_id);
        let session = Session {id: session_id, hash: session_hash};
        println!("Successfully created session: {:?}", session);
        self.sessions.push(session.clone());
        Ok(session)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    id: u32,
    hash: String
}

pub struct SessionWriter {
    session: Session,
    writer: EventWriter
}

impl SessionWriter {
    pub fn write_to_session(&mut self, topic: String, msg: String) -> Result<(), String> {
        let event = Event {msg, key: self.session.id.to_string(), topic};
        self.writer.write(event)
    }
}

pub struct SessionReader {
    session: Session,
    reader: EventReader
}

impl SessionReader {
    pub fn read_from_session(&mut self, topic: String) -> Result<Vec<Event>, String> {
        self.reader.read_from_topic(topic.as_str(), &self.session.id.to_string())
    }
}
