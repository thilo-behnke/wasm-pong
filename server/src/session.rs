use kafka::producer::Producer;
use crate::hash::Hasher;
use crate::kafka::{KafkaDefaultEventWriterImpl, KafkaSessionEventWriterImpl, KafkaTopicManager};
use serde::{Serialize, Deserialize};
use serde_json::json;
use pong::event::event::{Event, EventReader, EventWriter};

pub struct SessionManager {
    sessions: Vec<Session>,
    session_producer: EventWriter,
    topic_manager: KafkaTopicManager
}

impl SessionManager {
    pub fn new(kafka_host: &str) -> SessionManager {
        SessionManager {
            sessions: vec![],
            topic_manager: KafkaTopicManager::from("localhost:7243"),
            session_producer: EventWriter::new(Box::new(KafkaDefaultEventWriterImpl::new(kafka_host)))
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
        let json_event = serde_json::to_string(&SessionEvent::created(session.clone())).unwrap();
        let session_event_write = self.session_producer.write(Event {topic: "session".to_owned(), key: None, msg: json_event});
        if let Err(e) = session_event_write {
            let message = format!("Failed to write session create event to kafka: {:?}", e);
            println!("{}", e);
            return Err(message.to_owned());
        }
        println!("Successfully produced session event.");
        self.sessions.push(session.clone());
        Ok(session)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        let event = Event {msg, key: Some(self.session.id.to_string()), topic};
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

#[derive(Deserialize, Serialize)]
struct SessionEvent {
    event_type: SessionEventType,
    session: Session
}

impl SessionEvent {
    pub fn created(session: Session) -> SessionEvent {
        SessionEvent {event_type: SessionEventType::CREATED, session}
    }
}

#[derive(Deserialize, Serialize)]
enum SessionEventType {
    CREATED
}
