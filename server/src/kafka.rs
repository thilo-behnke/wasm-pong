use std::hash::{BuildHasher, Hash};
use std::process::ExitStatus;
use std::str::FromStr;
use std::time::Duration;
use hyper::{Body, Client, Method, Request, Uri};
use serde::{Deserialize};
use kafka::client::{KafkaClient, ProduceMessage};
use kafka::client::metadata::Topic;
use tokio::process::Command;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage, MessageSet};
use kafka::producer::{DefaultPartitioner, Partitioner, Producer, Record, RequiredAcks, Topics};
use pong::event::event::{Event, EventReaderImpl, EventWriter, EventWriterImpl};
use crate::hash::Hasher;
use crate::session::Session;

pub struct KafkaEventWriterImpl {
    producer: Producer<SessionPartitioner>
}
impl KafkaEventWriterImpl {
    pub fn default() -> KafkaEventWriterImpl {
        KafkaEventWriterImpl::new("localhost:9093")
    }

    pub fn from(host: &str) -> KafkaEventWriterImpl {
        KafkaEventWriterImpl::new(host)
    }

    pub fn new(host: &str) -> KafkaEventWriterImpl {
        println!("Connecting producer to kafka host: {}", host);
        let mut producer = Producer::from_hosts(vec![host.to_owned()])
            .with_ack_timeout(Duration::from_secs(1))
            .with_required_acks(RequiredAcks::One)
            .with_partitioner(SessionPartitioner {})
            .create()
            .unwrap();
        KafkaEventWriterImpl {
            producer
        }
    }
}
impl EventWriterImpl for KafkaEventWriterImpl {
    fn write(&mut self, event: Event) -> Result<(), String> {
        let record = Record::from_key_value(event.topic.as_str(), event.key.as_str(), event.msg.as_str());
        match self.producer.send(&record) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("{}", e))
        }
    }
}

pub struct KafkaEventReaderImpl {
    consumer: Consumer
}
impl KafkaEventReaderImpl {
    pub fn default() -> KafkaEventReaderImpl {
        KafkaEventReaderImpl::new("localhost:9093")
    }

    pub fn from(host: &str) -> KafkaEventReaderImpl {
        KafkaEventReaderImpl::new(host)
    }

    pub fn new(host: &str) -> KafkaEventReaderImpl {
        println!("Connecting consumer to kafka host: {}", host);
        let mut consumer = Consumer::from_hosts(vec!(host.to_owned()))
            .with_topic("move".to_owned())
            .with_topic("status".to_owned())
            .with_topic("input".to_owned())
            .with_fallback_offset(FetchOffset::Earliest)
            .with_group("group".to_owned())
            .with_offset_storage(GroupOffsetStorage::Kafka)
            .create()
            .unwrap();
        KafkaEventReaderImpl {
            consumer
        }
    }
}
impl EventReaderImpl for KafkaEventReaderImpl {
    fn read(&mut self) -> Result<Vec<Event>, String> {
        self.consume(None, None)
    }

    fn read_from_topic(&mut self, topic: &str, key: &str) -> Result<Vec<Event>, String> {
        self.consume(Some(topic), Some(key))
    }
}

impl KafkaEventReaderImpl {
    fn consume(&mut self, topic: Option<&str>, key: Option<&str>) -> Result<Vec<Event>, String> {
        // TODO: How to best filter messages by key (= game session id?)
        // E.g. https://docs.rs/kafka/latest/kafka/producer/struct.DefaultPartitioner.html - is it possible to read from partition by retrieving the hash of the key?
        // Does it even make sense to hash the key if it already is a hash? Custom partitioner?
        let polled = self.consumer.poll().unwrap();
        let message_sets: Vec<MessageSet<'_>> = polled.iter().collect();
        let mut events = vec![];
        for ms in message_sets {
            let topic = ms.topic();
            let partition = ms.partition();
            println!("querying topic={} partition={}", topic, partition);
            for m in ms.messages() {
                let event = Event {topic: String::from(topic), key: std::str::from_utf8(m.key).unwrap().parse().unwrap(), msg: std::str::from_utf8(m.value).unwrap().parse().unwrap() };
                events.push(event);
            }
            self.consumer.consume_messageset(ms).unwrap();
        }
        self.consumer.commit_consumed().unwrap();
        Ok(events)
    }
}

#[derive(Debug)]
pub struct KafkaTopicManager {
    partition_management_endpoint: String
}
impl KafkaTopicManager {

    pub fn default() -> KafkaTopicManager {
        KafkaTopicManager {partition_management_endpoint: "http://localhost:7243/add_partition".to_owned()}
    }

    pub fn from(topic_manager_host: &str) -> KafkaTopicManager {
        KafkaTopicManager {partition_management_endpoint: format!("http://{}/add_partition", topic_manager_host).to_owned()}
    }

    pub async fn add_partition(&self) -> Result<u32, String> {
        let mut client = Client::new();
        let request = Request::builder().method(Method::POST).uri(Uri::from_str(&self.partition_management_endpoint).unwrap()).body(Body::empty()).unwrap();
        let res = client.request(request).await;
        if let Err(e) = res {
            let error = format!("Failed to add partition: {:?}", e);
            println!("{}", error);
            return Err(error);
        }
        let status = res.as_ref().unwrap().status();
        let bytes = hyper::body::to_bytes(res.unwrap()).await;
        if let Err(e) = bytes {
            let error = format!("Failed to read bytes from response: {:?}", e);
            println!("{}", error);
            return Err(error);
        }
        let bytes = bytes.unwrap().to_vec();
        let res_str = std::str::from_utf8(&*bytes);
        if let Err(e) = res_str {
            let error = format!("Failed to deserialize bytes to string: {:?}", e);
            println!("{}", error);
            return Err(error);
        }
        if status != 200 {
            let error = format!("Failed to add partition: {}", res_str.unwrap());
            println!("{}", error);
            return Err(error);
        }
        let json = serde_json::from_str::<PartitionApiDTO>(res_str.unwrap());
        if let Err(e) = json {
            let error = format!("Failed to convert string {} to json: {:?}", res_str.unwrap(), e);
            println!("{}", error);
            return Err(error);
        }
        let updated_partition_count = json.unwrap().data;
        println!("Successfully created partition: {}", updated_partition_count);
        Ok(updated_partition_count)
    }
}

#[derive(Deserialize)]
struct PartitionApiDTO {
    data: u32
}

pub struct SessionPartitioner {}

impl Partitioner for SessionPartitioner {
    fn partition(&mut self, topics: Topics, msg: &mut ProduceMessage) {
        match msg.key {
            Some(key) => {
                let key = std::str::from_utf8(key).unwrap();
                msg.partition = key.parse::<i32>().unwrap();
                println!("Overriding message partition with key: {}", msg.partition);
            },
            None => panic!("Producing message without key not allowed!")
        }
    }
}
