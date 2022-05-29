use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use kafka::producer::Producer;
use crate::hash::Hasher;
use crate::kafka::{KafkaDefaultEventWriterImpl, KafkaEventReaderImpl, KafkaSessionEventReaderImpl, KafkaSessionEventWriterImpl, KafkaTopicManager};
use serde::{Serialize, Deserialize};
use serde_json::json;
use tokio::sync::Mutex;
use pong::event::event::{Event, EventReader, EventWriter};

pub struct SessionManager {
    kafka_host: String,
    sessions: Vec<Session>,
    session_producer: EventWriter,
    topic_manager: KafkaTopicManager
}

// TODO: On startup read the session events from kafka to restore the session id <-> hash mappings.
impl SessionManager {
    pub fn new(kafka_host: &str) -> SessionManager {
        SessionManager {
            kafka_host: kafka_host.to_owned(),
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

    pub fn get_session_reader(&self, session_id: &str) -> Result<SessionReader, String> {
        let session = self.find_session(&session_id);
        if let None = session {
            return Err(format!("Unable to find session with hash {}", session_id))
        }
        let session = session.unwrap();
        let event_reader = EventReader::new(Box::new(KafkaSessionEventReaderImpl::new(&self.kafka_host, &session, &["move", "status", "input"])));
        Ok(SessionReader {reader: event_reader, session})
    }

    pub fn get_session_writer(&self, session_id: &str) -> Result<SessionWriter, String> {
        let session = self.find_session(&session_id);
        if let None = session {
            return Err(format!("Unable to find session with hash {}", session_id))
        }
        let session = session.unwrap();
        let event_writer = EventWriter::new(Box::new(KafkaSessionEventWriterImpl::new(&self.kafka_host)));
        Ok(SessionWriter {writer: event_writer, session})
    }

    fn find_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.iter().find(|s| session_id == s.hash).map(|s| s.clone())
    }
}

pub struct CachingSessionManager {
    inner: SessionManager,
    reader_cache: HashMap<String, Arc<Mutex<SessionReader>>>,
    writer_cache: HashMap<String, Arc<Mutex<SessionWriter>>>,
}

impl CachingSessionManager {
    pub fn new(kafka_host: &str) -> CachingSessionManager {
        CachingSessionManager {
            inner: SessionManager::new(kafka_host),
            reader_cache: HashMap::new(),
            writer_cache: HashMap::new(),
        }
    }

    pub async fn create_session(&mut self) -> Result<Session, String> {
        self.inner.create_session().await
    }

    pub fn get_session_reader(&mut self, session_id: &str) -> Result<Arc<Mutex<SessionReader>>, String> {
        let cached = self.reader_cache.get(session_id);
        if let Some(reader) = cached {
            println!("Reusing existing reader for session: {:?}", session_id);
            return Ok(Arc::clone(reader));
        }
        let reader = self.inner.get_session_reader(session_id);
        if let Err(e) = reader {
            return Err(e);
        }
        let reader = Arc::new(Mutex::new(reader.unwrap()));
        self.reader_cache.insert(session_id.to_string(), Arc::clone(&reader));
        return Ok(Arc::clone(&reader));
    }

    pub fn get_session_writer(&mut self, session_id: &str) -> Result<Arc<Mutex<SessionWriter>>, String> {
        let cached = self.writer_cache.get(session_id);
        if let Some(writer) = cached {
            println!("Reusing existing writer for session: {:?}", session_id);
            return Ok(Arc::clone(writer));
        }
        let writer = self.inner.get_session_writer(session_id);
        if let Err(e) = writer {
            return Err(e);
        }
        let writer = Arc::new(Mutex::new(writer.unwrap()));
        self.writer_cache.insert(session_id.to_string(), Arc::clone(&writer));
        return Ok(Arc::clone(&writer));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: u16,
    pub hash: String
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
    pub fn read_from_session(&mut self) -> Result<Vec<Event>, String> {
        self.reader.read()
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
