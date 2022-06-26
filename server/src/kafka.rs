use std::str::FromStr;
use std::time::Duration;
use futures::future::err;

use hyper::{Body, Client, Method, Request, Uri};
use kafka::client::ProduceMessage;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage, MessageSet};
use kafka::producer::{Partitioner, Producer, Record, RequiredAcks, Topics};
use log::{debug, error, info, trace};
use serde::Deserialize;

use pong::event::event::{Event, EventReaderImpl, EventWriterImpl};
use crate::session::Session;

pub struct KafkaSessionEventWriterImpl {
    producer: Producer<SessionPartitioner>,
}

impl KafkaSessionEventWriterImpl {
    pub fn new(host: &str) -> KafkaSessionEventWriterImpl {
        info!("Connecting session_writer producer to kafka host: {}", host);
        let producer = Producer::from_hosts(vec![host.to_owned()])
            .with_ack_timeout(Duration::from_secs(1))
            .with_required_acks(RequiredAcks::One)
            .with_partitioner(SessionPartitioner {})
            .create();
        if let Err(ref e) = producer {
            error!("Failed to connect kafka producer: {:?}", e)
        }
        let producer = producer.unwrap();
        KafkaSessionEventWriterImpl { producer }
    }
}

pub struct KafkaDefaultEventWriterImpl {
    producer: Producer,
}

impl KafkaDefaultEventWriterImpl {
    pub fn new(host: &str) -> KafkaDefaultEventWriterImpl {
        info!("connecting default producer to kafka host: {}", host);
        let producer = Producer::from_hosts(vec![host.to_owned()])
            .with_ack_timeout(Duration::from_secs(1))
            .with_required_acks(RequiredAcks::One)
            .create();
        if let Err(e) = producer {
            error!("failed to connect producer to kafka host {}: {:?}", host, e);
            panic!("kafka connection failed, no recovery possible.")
        }
        let producer = producer.unwrap();
        KafkaDefaultEventWriterImpl { producer }
    }
}

impl EventWriterImpl for KafkaSessionEventWriterImpl {
    fn write(&mut self, events: Vec<Event>) -> Result<(), String> {
        write_events(events, &mut self.producer)
    }
}

impl EventWriterImpl for KafkaDefaultEventWriterImpl {
    fn write(&mut self, events: Vec<Event>) -> Result<(), String> {
        write_events(events, &mut self.producer)
    }
}

fn write_events<T>(events: Vec<Event>, producer: &mut Producer<T>) -> Result<(), String> where T : Partitioner {
    let mut records_without_key = vec![];
    let mut records_with_key = vec![];
    for event in events.iter() {
        match &event.key {
            Some(key) => {
                let record = Record::from_key_value(&event.topic, key.clone(), event.msg.clone());
                records_with_key.push(record);
            }
            None => {
                let record = Record::from_value(&event.topic, event.msg.clone());
                records_without_key.push(record);
            }
        }
    }

    let res_with_key = match producer.send_all::<String, String>(&*records_with_key) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{}", e)),
    };
    let res_without_key = match producer.send_all::<(), String>(&*records_without_key) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{}", e)),
    };
    res_with_key.and(res_without_key)
}

pub struct KafkaEventReaderImpl {
    consumer: Consumer,
    topics: Vec<String>,
    partitions: Vec<i32>
}

impl KafkaEventReaderImpl {
    pub fn for_partitions(
        host: &str,
        partitions: &[i32],
        topics: &[&str],
    ) -> Result<KafkaEventReaderImpl, String> {
        debug!("connecting partition specific consumer to kafka host {} with topics {:?} / partitions {:?}", host, topics, partitions);
        let mut builder = Consumer::from_hosts(vec![host.to_owned()]);
        let topics = topics.iter().map(|s| s.to_owned().to_owned()).collect::<Vec<String>>();
        let partitions = partitions.iter().map(|i| *i).collect::<Vec<i32>>();
        for topic in topics.iter() {
            builder = builder.with_topic_partitions(topic.parse().unwrap(), &*partitions);
        }
        builder = builder
            .with_fallback_offset(FetchOffset::Earliest)
            .with_group("group".to_owned())
            .with_offset_storage(GroupOffsetStorage::Kafka);

        let consumer = builder.create();
        if let Err(e) = consumer {
            let error = format!("Failed to connect consumer: {:?}", e);
            error!("{}", error);
            return Err(error);
        }
        let consumer = consumer.unwrap();
        debug!("successfully connected partition specific consumer to kafka host {} with topics {:?} / partitions {:?}", host, topics, partitions);
        Ok(KafkaEventReaderImpl { consumer, topics, partitions })
    }
}

impl EventReaderImpl for KafkaEventReaderImpl {
    fn read(&mut self) -> Result<Vec<Event>, String> {
        self.consume()
    }
}

impl KafkaEventReaderImpl {
    fn consume(&mut self) -> Result<Vec<Event>, String> {
        debug!("kafka consumer called to consume messages for {:?} / {:?}", self.topics, self.partitions);
        let polled = self.consumer.poll().unwrap();
        let message_sets: Vec<MessageSet<'_>> = polled.iter().collect();
        let mut events = vec![];
        for ms in message_sets {
            let mut topic_event_count = 0;
            let topic = ms.topic();
            let partition = ms.partition();
            trace!("querying kafka topic={} partition={}", topic, partition);
            for m in ms.messages() {
                let event = Event {
                    topic: String::from(topic),
                    key: Some(std::str::from_utf8(m.key).unwrap().parse().unwrap()),
                    msg: std::str::from_utf8(m.value).unwrap().parse().unwrap(),
                };
                topic_event_count += 1;
                events.push(event);
            }
            trace!(
                "returned {:?} events for topic={} partition={}",
                topic_event_count, topic, partition
            );
            self.consumer.consume_messageset(ms).unwrap();
        }
        self.consumer.commit_consumed().unwrap();
        trace!("kafka consumed {} messages for {:?} / {:?}", events.len(), self.topics, self.partitions);
        Ok(events)
    }
}

pub struct KafkaSessionEventReaderImpl {
    inner: KafkaEventReaderImpl,
}

impl KafkaSessionEventReaderImpl {
    pub fn new(
        host: &str,
        session: &Session,
        topics: &[&str],
    ) -> Result<KafkaSessionEventReaderImpl, String> {
        let partitions = [session.id as i32];
        let reader = KafkaEventReaderImpl::for_partitions(host, &partitions, topics);
        if let Err(_) = reader {
            return Err("Failed to create kafka session event reader".to_string());
        }
        let reader = reader.unwrap();
        Ok(KafkaSessionEventReaderImpl { inner: reader })
    }
}

impl EventReaderImpl for KafkaSessionEventReaderImpl {
    fn read(&mut self) -> Result<Vec<Event>, String> {
        self.inner.read()
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

pub struct SessionPartitioner {}

impl Partitioner for SessionPartitioner {
    fn partition(&mut self, _topics: Topics, msg: &mut ProduceMessage) {
        match msg.key {
            Some(key) => {
                let key = std::str::from_utf8(key).unwrap();
                msg.partition = key.parse::<i32>().unwrap();
                // println!("Overriding message partition with key: {}", msg.partition);
            }
            None => panic!("Producing message without key not allowed!"),
        }
    }
}
