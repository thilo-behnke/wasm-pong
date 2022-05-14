use std::process::ExitStatus;
use std::time::Duration;
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
pub struct KafkaTopicManager {}
impl KafkaTopicManager {
    pub async fn add_partition(&self) -> Result<u32, String> {
        let topics = vec!["move", "status", "input"];
        let mut client = KafkaClient::new(vec!["localhost:9092".to_owned()]);
        client.load_metadata_all().unwrap();
        let topic_metas: Vec<Topic> = client.topics().into_iter().filter(|t| topics.contains(&t.name())).collect();
        if topic_metas.len() != 3 {
            return Err(format!("Can't add_partition, unable to find matching topics: {:?}", topics));
        }
        let max_partition_count = topic_metas.into_iter().map(|t| t.partitions().len()).max().unwrap();
        let next_partition = max_partition_count + 1;
        println!("Will create next partition: {}", next_partition);
        // TODO: What if creating one of the partitions fails?
        for topic in topics {
            let output = Command::new("/bin/bash").arg(format!("-c bin/kafka-topics.sh --bootstrap-server localhost:9092 --alter --topic {} --partitions {}", topic, next_partition)).output().await;
            if let Err(e) = output {
                return Err(format!("{}", e))
            }
            let output = output.unwrap();
            if !output.status.success() {
                return Err(format!("{:?}", std::str::from_utf8(&*output.stderr)))
            }
        }
        Ok(next_partition as u32)
    }
}
