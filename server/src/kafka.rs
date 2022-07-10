use async_trait::async_trait;
use std::collections::{BTreeMap, HashMap};
use std::fs::read;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use futures::{StreamExt, TryFutureExt};
use futures::future::err;

use hyper::{Body, Client, Method, Request, Uri};
use log::{debug, error, info, trace};
use rskafka::client::ClientBuilder;
use rskafka::client::consumer::{StartOffset, StreamConsumer, StreamConsumerBuilder};
use rskafka::client::partition::{Compression, PartitionClient};
use rskafka::record::Record;
use rskafka::time::OffsetDateTime;
use serde::Deserialize;

use pong::event::event::{EventWrapper, EventReaderImpl, EventWriterImpl};
use crate::session::Session;

pub struct KafkaSessionEventWriterImpl {
    topics: Vec<String>,
    partitions: Vec<i32>,
    producers: HashMap<String, PartitionClient>
}

impl KafkaSessionEventWriterImpl {
    pub async fn new(host: &str, topics: Vec<&str>, partition: &i32) -> KafkaSessionEventWriterImpl {
        info!("Connecting session_writer producer to kafka host: {}", host);
        let owned_topics = topics.iter().map(|t| t.to_owned().to_owned()).collect();
        let mut producers = HashMap::new();
        for topic in topics {
            let client = ClientBuilder::new(vec![host.to_owned()]).build().await.unwrap();
            let producer = client.partition_client(topic.to_owned(), partition.clone()).await;
            if let Err(ref e) = producer {
                error!("Failed to connect kafka producer: {:?}", e)
            }
            let producer = producer.unwrap();
            producers.insert(topic.to_owned(), producer);
        }
        KafkaSessionEventWriterImpl { topics: owned_topics, partitions: vec![partition.clone()], producers }
    }
}

#[async_trait]
impl EventWriterImpl for KafkaSessionEventWriterImpl {
    async fn write(&mut self, events: Vec<EventWrapper>) -> Result<(), String> {
        let mut by_topic: HashMap<String, Vec<EventWrapper>> = HashMap::new();
        for e in events {
            match by_topic.get_mut(&e.topic) {
                Some(events) => events.push(e),
                None => {
                    let mut events = vec![];
                    let topic = e.topic.clone();
                    events.push(e);
                    by_topic.insert(topic, events);
                }
            }
        }
        for topic_events in by_topic {
            let mut producer = self.producers.get_mut(&topic_events.0);
            if let None = producer {
                let available = self.producers.keys().collect::<Vec<&String>>();
                return Err(format!("Could not find producer for topic: {}. Available topic producers: {:?}", &topic_events.0, available));
            }
            let producer = producer.unwrap();
            let res = write_events(topic_events.1, producer).await;
            if let Err(e) = res {
                return Err(e);
            }
        }
        Ok(())
    }
}

async fn write_events(events: Vec<EventWrapper>, producer: &mut PartitionClient) -> Result<(), String> {
    let mut records = vec![];
    for event in events.iter() {
        match &event.key {
            Some(key) => {
                let key = Some(key.clone().into_bytes());
                let value = Some(event.event.clone().into_bytes());
                let headers = BTreeMap::new();
                let timestamp = OffsetDateTime::now_utc();
                let record = Record { key, value, headers, timestamp };
                records.push(record);
            }
            None => {
                let key = None;
                let value = Some(event.event.clone().into_bytes());
                let headers = BTreeMap::new();
                let timestamp = OffsetDateTime::now_utc();
                let record = Record { key, value, headers, timestamp };
                records.push(record);
            }
        }
    }

    let res = match producer.produce(records, Compression::NoCompression).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{:?}", e)),
    };
    res
}

pub struct KafkaEventReaderImpl {
    consumer: StreamConsumer,
    topic: String,
    partition: i32
}

// TODO: Hotfix, but does this really work?
unsafe impl Send for KafkaEventReaderImpl {}
unsafe impl Sync for KafkaEventReaderImpl {}

impl KafkaEventReaderImpl {
    pub async fn for_partition(
        host: &str,
        partition: &i32,
        topic: &str,
    ) -> Result<KafkaEventReaderImpl, String> {
        debug!("connecting partition specific consumer to kafka host {} with topic {:?} / partition {:?}", host, topic, partition);
        let partition_client = ClientBuilder::new(vec![host.to_owned()]).build().await.unwrap()
            .partition_client(topic.to_owned(), partition.clone()).await;

        if let Err(e) = partition_client {
            let error = format!("Failed to connect consumer: {:?}", e);
            error!("{}", error);
            return Err(error);
        }
        let partition_client = Arc::new(partition_client.unwrap());
        let consumer = StreamConsumerBuilder::new(
            partition_client,
            StartOffset::Earliest
        ).with_max_wait_ms(1).build();
        debug!("successfully connected partition specific consumer to kafka host {} with topic {:?} / partition {:?}", host, topic, partition);
        Ok(KafkaEventReaderImpl { consumer, topic: topic.to_owned(), partition: partition.clone() })
    }
}

#[async_trait]
impl EventReaderImpl for KafkaEventReaderImpl {
    async fn read(&mut self) -> Result<Vec<EventWrapper>, String> {
        self.consume().await
    }
}

impl KafkaEventReaderImpl {
    async fn consume(&mut self) -> Result<Vec<EventWrapper>, String> {
        debug!("kafka consumer called to consume messages for {:?} / {:?}", self.topic, self.partition);
        // TODO: Only 1 message?
        let next_res = tokio::time::timeout(Duration::from_millis(3), self.consumer.next()).await;
        if let Err(e) = next_res {
            info!("No record received in time after {}, timeout for {} / {}.", e, self.topic, self.partition);
            return Ok(vec![]);
        }
        let next_res = next_res.unwrap();
        if let None = next_res {
            debug!("No record retrieved for {} / {}", self.topic, self.partition);
            return Err("No record.".to_owned());
        }
        let next_res = next_res.unwrap();
        if let Err(e) = next_res {
            let error = format!("Failed to extract record for {} / {}: {:?}", self.topic, self.partition, e);
            error!("{}", error);
            return Err(error);
        }
        let (record, _) = next_res.unwrap();
        debug!("kafka consumer retrieved record: {:?}", record);
        let key = match record.record.key {
            Some(k) => Some(std::str::from_utf8(&*k).unwrap().to_owned()),
            None => None
        };
        let event = match record.record.value {
            Some(e) => std::str::from_utf8(&*e).unwrap().to_owned(),
            None => panic!("event without payload!")
        };
        let event = EventWrapper {
            topic: String::from(self.topic.clone()),
            key,
            event,
        };
        debug!("converted record to event: {:?}", event);
        let events = vec![event];
        Ok(events)
    }
}

pub struct KafkaSessionEventReaderImpl {
    inner: HashMap<String, KafkaEventReaderImpl>
}

impl KafkaSessionEventReaderImpl {
    pub async fn new(
        host: &str,
        session: &Session,
        topics: &[&str],
    ) -> Result<KafkaSessionEventReaderImpl, String> {
        let mut reader_map = HashMap::new();
        for topic in topics {
            let reader = KafkaEventReaderImpl::for_partition(host, &i32::from(session.id), *topic).await;
            if let Err(_) = reader {
                return Err("Failed to create kafka session event reader".to_string());
            }
            let reader = reader.unwrap();
            reader_map.insert(topic.to_owned().to_owned(), reader);
        }
        Ok(KafkaSessionEventReaderImpl { inner: reader_map })
    }
}

#[async_trait]
impl EventReaderImpl for KafkaSessionEventReaderImpl {
    async fn read(&mut self) -> Result<Vec<EventWrapper>, String> {
        let mut events = vec![];
        for topic_reader in self.inner.iter_mut() {
            let topic_events = topic_reader.1.read().await;
            if let Err(e) = topic_events {
                let error = format!("Failed to consume events for topic {}: {}", topic_reader.0, e);
                return Err(error);
            }
            let topic_events = topic_events.unwrap();
            for event in topic_events {
                events.push(event);
            }
        }
        Ok(events)
    }
}

#[derive(Debug)]
pub struct KafkaTopicManager {
    partition_management_endpoint: String,
}

impl KafkaTopicManager {
    pub fn from(topic_manager_host: &str) -> KafkaTopicManager {
        KafkaTopicManager {
            partition_management_endpoint: format!("http://{}/add_partition", topic_manager_host)
                .to_owned(),
        }
    }

    pub async fn add_partition(&self) -> Result<u16, String> {
        debug!("called to create new partition");
        let client = Client::new();
        let request = Request::builder()
            .method(Method::POST)
            .uri(Uri::from_str(&self.partition_management_endpoint).unwrap())
            .body(Body::empty())
            .unwrap();
        let res = client.request(request).await;
        if let Err(e) = res {
            let error = format!("failed to add partition: {:?}", e);
            error!("{}", error);
            return Err(error);
        }
        let status = res.as_ref().unwrap().status();
        let bytes = hyper::body::to_bytes(res.unwrap()).await;
        if let Err(e) = bytes {
            let error = format!("failed to read bytes from response: {:?}", e);
            println!("{}", error);
            return Err(error);
        }
        let bytes = bytes.unwrap().to_vec();
        let res_str = std::str::from_utf8(&*bytes);
        if let Err(e) = res_str {
            let error = format!("failed to deserialize bytes to string: {:?}", e);
            println!("{}", error);
            return Err(error);
        }
        if status != 200 {
            let error = format!("failed to add partition: {}", res_str.unwrap());
            println!("{}", error);
            return Err(error);
        }
        let json = serde_json::from_str::<PartitionApiDTO>(res_str.unwrap());
        if let Err(e) = json {
            let error = format!(
                "failed to convert string {} to json: {:?}",
                res_str.unwrap(),
                e
            );
            println!("{}", error);
            return Err(error);
        }
        let updated_partition_count = json.unwrap().data;
        debug!(
            "successfully created partition: {}",
            updated_partition_count
        );
        Ok(updated_partition_count)
    }
}

#[derive(Deserialize)]
struct PartitionApiDTO {
    data: u16,
}
