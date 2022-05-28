use std::process::ExitStatus;
use std::str::FromStr;
use std::time::Duration;
use hyper::{Client, Uri};
use serde::{Deserialize};
use kafka::client::KafkaClient;
use kafka::client::metadata::Topic;
use tokio::process::Command;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage, MessageSet};
use kafka::producer::{DefaultPartitioner, Producer, Record, RequiredAcks, Topics};
use pong::event::event::{Event, EventReaderImpl, EventWriter, EventWriterImpl};

pub struct KafkaEventWriterImpl {
    producer: Producer
}
impl KafkaEventWriterImpl {
    pub fn default() -> KafkaEventWriterImpl {
        KafkaEventWriterImpl::new("localhost:9092")
    }

    pub fn new(host: &str) -> KafkaEventWriterImpl {
        let mut producer = Producer::from_hosts(vec![host.to_owned()])
            .with_ack_timeout(Duration::from_secs(1))
            .with_required_acks(RequiredAcks::One)
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
        KafkaEventReaderImpl::new("localhost:9092")
    }

    pub fn new(host: &str) -> KafkaEventReaderImpl {
        let mut consumer = Consumer::from_hosts(vec![host.to_owned()])
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
    fn read(&mut self) -> Vec<Event> {
        self.consume(None, None)
    }

    fn read_from_topic(&mut self, topic: &str, key: &str) -> Vec<Event> {
        self.consume(Some(topic), Some(key))
    }
}

impl KafkaEventReaderImpl {
    fn consume(&mut self, topic: Option<&str>, key: Option<&str>) -> Vec<Event> {
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
        events
    }
}

#[derive(Debug)]
pub struct KafkaTopicManager {
    partition_management_endpoint: String
}
impl KafkaTopicManager {

    pub fn default() -> KafkaTopicManager {
        KafkaTopicManager {partition_management_endpoint: "kafka:7243/add_partition".to_owned()}
    }

    pub async fn add_partition(&self) -> Result<u32, String> {
        let mut client = Client::new();
        let res = client.get(Uri::from_str(&self.partition_management_endpoint).unwrap()).await;
        if let Err(e) = res {
            let error = format!("Failed to add partition: {:?}", e);
            println!("{}", error);
            return Err(error);
        }
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
        let json = serde_json::from_str::<PartitionApiDTO>(res_str.unwrap());
        if let Err(e) = json {
            let error = format!("Failed to convert string to json: {:?}", e);
            println!("{}", error);
            return Err(error);
        }
        Ok(json.unwrap().data)
    }
}

#[derive(Deserialize)]
struct PartitionApiDTO {
    data: u32
}
