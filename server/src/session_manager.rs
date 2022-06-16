use serde::{Deserialize, Serialize};

use pong::event::event::{Event, EventReader, EventWriter};

use crate::hash::Hasher;
use crate::kafka::{
    KafkaDefaultEventWriterImpl, KafkaSessionEventReaderImpl, KafkaSessionEventWriterImpl,
    KafkaTopicManager,
};
use crate::player::Player;
use crate::session::{Session, SessionState};

pub struct SessionManager {
    kafka_host: String,
    sessions: Vec<Session>,
    session_producer: EventWriter,
    topic_manager: KafkaTopicManager,
}

// TODO: On startup read the session events from kafka to restore the session id <-> hash mappings.
impl SessionManager {
    pub fn new(kafka_host: &str, kafka_topic_manager_host: &str) -> SessionManager {
        SessionManager {
            kafka_host: kafka_host.to_owned(),
            sessions: vec![],
            topic_manager: KafkaTopicManager::from(kafka_topic_manager_host),
            session_producer: EventWriter::new(Box::new(KafkaDefaultEventWriterImpl::new(
                kafka_host,
            ))),
        }
    }

    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions
            .iter()
            .find(|s| s.hash == session_id)
            .map_or_else(|| None, |s| Some(s.clone()))
    }

    pub async fn create_session(&mut self, player: Player) -> Result<Session, String> {
        let add_partition_res = self.topic_manager.add_partition().await;
        if let Err(e) = add_partition_res {
            println!("Failed to create partition: {}", e);
            return Err(e);
        }
        let session_id = add_partition_res.unwrap();
        let session_hash = Hasher::hash(session_id);
        let session = Session::new(session_id, session_hash, player.clone());
        println!("Successfully created session: {:?}", session);
        let write_res = self.write_to_producer(session_created(session.clone(), player.clone()));
        if let Err(e) = write_res {
            eprintln!(
                "Failed to write session created event for {:?} to producer: {}",
                session, e
            );
        }
        self.sessions.push(session.clone());
        Ok(session)
    }

    pub async fn join_session(
        &mut self,
        session_id: String,
        player: Player,
    ) -> Result<Session, String> {
        let updated_session = {
            let session = self.sessions.iter_mut().find(|s| s.hash == session_id);
            if let None = session {
                let error = format!("Can't join session that does not exist: {}", session_id);
                return Err(error);
            }
            let mut session = session.unwrap();
            if session.state != SessionState::PENDING {
                let error = format!("Can't join session that is not PENDING: {}", session_id);
                return Err(error);
            }
            if session.players.len() > 1 {
                let error = format!("Can't join session with more than 1 player: {}", session_id);
                return Err(error);
            }
            if session.players[0] == player {
                let error = format!(
                    "Can't join session, because player {:?} is already in session: {}",
                    player, session_id
                );
                return Err(error);
            }
            session.players.push(player.clone());
            session.state = SessionState::RUNNING;
            session.clone()
        };
        {
            let write_res =
                self.write_to_producer(session_joined(updated_session.clone(), player.clone()));
            if let Err(e) = write_res {
                eprintln!(
                    "Failed to write session joined event for {:?} to producer: {}",
                    updated_session, e
                );
            }
        };
        println!("sessions = {:?}", self.sessions);
        Ok(updated_session.clone())
    }

    fn write_to_producer<T>(&mut self, session_event: T) -> Result<(), String>
    where
        T: Serialize,
    {
        let json_event = serde_json::to_string(&session_event).unwrap();
        let session_event_write = self.session_producer.write(Event {
            topic: "session".to_owned(),
            key: None,
            msg: json_event,
        });
        if let Err(e) = session_event_write {
            let message = format!("Failed to write session create event to kafka: {:?}", e);
            println!("{}", e);
            return Err(message.to_owned());
        }
        println!("Successfully produced session event.");
        return Ok(());
    }

    pub fn split(
        &self,
        session_id: &str,
        read_topics: &[&str],
    ) -> Result<(SessionReader, SessionWriter), String> {
        let reader = self.get_session_reader(session_id, read_topics);
        if let Err(e) = reader {
            println!("Failed to create session reader: {:?}", e);
            return Err("Failed to create session reader".to_string());
        }
        let writer = self.get_session_writer(session_id);
        if let Err(e) = writer {
            println!("Failed to create session writer: {:?}", e);
            return Err("Failed to create session writer".to_string());
        }
        return Ok((reader.unwrap(), writer.unwrap()));
    }

    pub fn get_session_reader(
        &self,
        session_id: &str,
        topics: &[&str],
    ) -> Result<SessionReader, String> {
        let session = self.find_session(&session_id);
        if let None = session {
            return Err(format!("Unable to find session with hash {}", session_id));
        }
        let session = session.unwrap();
        let kafka_reader = KafkaSessionEventReaderImpl::new(&self.kafka_host, &session, topics);
        if let Err(_) = kafka_reader {
            return Err("Unable to create kafka reader.".to_string());
        }
        let kafka_reader = kafka_reader.unwrap();
        let event_reader = EventReader::new(Box::new(kafka_reader));
        Ok(SessionReader {
            reader: event_reader,
            session,
        })
    }

    pub fn get_session_writer(&self, session_id: &str) -> Result<SessionWriter, String> {
        let session = self.find_session(&session_id);
        if let None = session {
            return Err(format!("Unable to find session with hash {}", session_id));
        }
        let session = session.unwrap();
        let event_writer =
            EventWriter::new(Box::new(KafkaSessionEventWriterImpl::new(&self.kafka_host)));
        Ok(SessionWriter {
            writer: event_writer,
            session,
        })
    }

    fn find_session(&self, session_id: &str) -> Option<Session> {
        self.sessions
            .iter()
            .find(|s| session_id == s.hash)
            .map(|s| s.clone())
    }
}

pub struct SessionWriter {
    session: Session,
    writer: EventWriter,
}

impl SessionWriter {
    pub fn write_to_session(&mut self, topic: &str, msg: &str) -> Result<(), String> {
        let event = Event {
            msg: msg.to_owned(),
            key: Some(self.session.id.to_string()),
            topic: topic.to_owned(),
        };
        self.writer.write(event)
    }
}

pub struct SessionReader {
    #[allow(dead_code)]
    session: Session,
    reader: EventReader,
}

impl SessionReader {
    pub fn read_from_session(&mut self) -> Result<Vec<Event>, String> {
        self.reader.read()
    }
}

#[derive(Deserialize, Serialize)]
struct SessionCreatedEvent {
    event_type: SessionEventType,
    session: Session,
    player: Player,
}

impl SessionCreatedEvent {
    pub fn new(session: Session, player: Player) -> SessionCreatedEvent {
        SessionCreatedEvent {
            event_type: SessionEventType::CREATED,
            session,
            player,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct SessionJoinedEvent {
    event_type: SessionEventType,
    session: Session,
    player: Player,
}

impl SessionJoinedEvent {
    pub fn new(session: Session, player: Player) -> SessionJoinedEvent {
        SessionJoinedEvent {
            event_type: SessionEventType::JOINED,
            session,
            player,
        }
    }
}

fn session_created(session: Session, player: Player) -> SessionCreatedEvent {
    SessionCreatedEvent::new(session, player)
}

fn session_joined(session: Session, player: Player) -> SessionJoinedEvent {
    SessionJoinedEvent::new(session, player)
}

#[derive(Deserialize, Serialize)]
enum SessionEventType {
    CREATED,
    JOINED,
}
