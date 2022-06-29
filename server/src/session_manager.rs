use std::collections::HashMap;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

use pong::event::event::{EventWrapper, EventReader, EventWriter};

use crate::hash::Hasher;
use crate::kafka::{
    KafkaSessionEventReaderImpl, KafkaSessionEventWriterImpl,
    KafkaTopicManager,
};
use crate::actor::Player;
use crate::event::{SessionEvent, SessionEventPayload};
use crate::session::{Session, SessionState};

pub struct SessionManager {
    kafka_host: String,
    sessions: Vec<Session>,
    session_producers: HashMap<String, SessionWriter>,
    topic_manager: KafkaTopicManager,
}

// TODO: On startup read the session events from kafka to restore the session id <-> hash mappings.
impl SessionManager {
    pub fn new(kafka_host: &str, kafka_topic_manager_host: &str) -> SessionManager {
        SessionManager {
            kafka_host: kafka_host.to_owned(),
            sessions: vec![],
            topic_manager: KafkaTopicManager::from(kafka_topic_manager_host),
            session_producers: HashMap::new(),
        }
    }

    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions
            .iter()
            .find(|s| s.session_id == session_id)
            .map_or_else(|| None, |s| Some(s.clone()))
    }

    pub async fn create_session(&mut self, player: Player) -> Result<SessionEvent, String> {
        info!("called to create new session by player {:?}", player);
        let add_partition_res = self.topic_manager.add_partition().await;
        if let Err(e) = add_partition_res {
            error!("failed to create partition: {}", e);
            return Err(e);
        }
        let session_partition_id = add_partition_res.unwrap();
        let session_id = Hasher::hash(session_partition_id);
        let session = Session::new(session_partition_id, session_id.clone(), player.clone());
        info!("successfully created session: {:?}", session);
        self.sessions.push(session.clone());
        let session_created = SessionEvent::Created(SessionEventPayload {
            session: session.clone(),
            actor: player,
            reason: format!("session created")
        });
        let write_res = self.write_to_producer(&session_created);
        if let Err(e) = write_res {
            let index = self.sessions.iter().position(|s| s.session_id == session_id);
            if let Some(i) = index {
                debug!("session create event could not be persisted - remove session from cache.");
                self.sessions.remove(i);
            }
            error!(
                "failed to write session created event for {:?} to producer: {}",
                session, e
            );
        }
        info!("successfully persisted create session event.");
        Ok(session_created)
    }

    pub async fn join_session(
        &mut self,
        session_id: String,
        player: Player,
    ) -> Result<SessionEvent, String> {
        let updated_session = {
            let session = self.sessions.iter_mut().find(|s| s.session_id == session_id);
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
        let session_joined_event = SessionEvent::Joined(SessionEventPayload {
            session: updated_session.clone(),
            reason: "session joined".to_owned(),
            actor: player
        });
        {
            let write_res =
                self.write_to_producer(&session_joined_event);
            if let Err(e) = write_res {
                eprintln!(
                    "Failed to write session joined event for {:?} to producer: {}",
                    updated_session, e
                );
            }
        };
        println!("sessions = {:?}", self.sessions);
        Ok(session_joined_event)
    }

    fn write_to_producer(&mut self, session_event: &SessionEvent) -> Result<(), String>
    {
        let session_id = session_event.session_id();
        let session_producer = match self.session_producers.get_mut(session_id) {
            Some(p) => p,
            None => {
                let session_writer = self.get_session_writer(session_id).expect("failed to create session writer to persist create event");
                self.session_producers.insert(session_id.to_owned(), session_writer);
                self.session_producers.get_mut(session_id).expect("failed to retrieve newly created session writer")
            }
        };
        let json_event = serde_json::to_string(&session_event);
        if let Err(e) = json_event {
            let error = format!("failed to serialize session event: {}", e);
            error!("{}", error);
            return Err(error);
        }
        let json_event = json_event.unwrap();
        info!("preparing to write session event to kafka: {}", json_event);
        let session_event_write = session_producer.write_to_session("session", vec![&json_event]);
        if let Err(e) = session_event_write {
            let message = format!("Failed to write session event to kafka: {:?}", e);
            println!("{}", e);
            return Err(message.to_owned());
        }
        info!("successfully produced session event: {:?}", json_event);
        return Ok(());
    }

    pub fn split(
        &self,
        session_id: &str,
        read_topics: &[&str],
    ) -> Result<(SessionReader, SessionWriter), String> {
        let reader = self.get_session_reader(session_id, read_topics);
        if let Err(e) = reader {
            error!("Failed to create session reader for session {}: {:?}", session_id, e);
            return Err("Failed to create session reader".to_string());
        }
        let writer = self.get_session_writer(session_id);
        if let Err(e) = writer {
            error!("Failed to create session writer for session {}: {:?}", session_id, e);
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
            .find(|s| session_id == s.session_id)
            .map(|s| s.clone())
    }
}

pub struct SessionWriter {
    session: Session,
    writer: EventWriter,
}

impl SessionWriter {
    pub fn write_to_session(&mut self, topic: &str, messages: Vec<&str>) -> Result<(), String> {
        let events = messages.into_iter().map(|e| {
            EventWrapper {
                event: e.to_owned(),
                key: Some(self.session.id.to_string()),
                topic: topic.to_owned(),
            }
        }).collect();
        self.writer.write_all(events)
    }
}

pub struct SessionReader {
    #[allow(dead_code)]
    session: Session,
    reader: EventReader,
}

impl SessionReader {
    pub fn read_from_session(&mut self) -> Result<Vec<EventWrapper>, String> {
        self.reader.read()
    }
}
