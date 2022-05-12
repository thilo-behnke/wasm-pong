use std::time::Duration;
use kafka::producer::{Producer, Record, RequiredAcks};
use pong::event::event::{Event, EventWriter, EventWriterImpl};

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
    fn write(&mut self, event: Event) -> Result<(), ()> {
        let record = Record::from_key_value(event.topic.as_str(), event.key.as_str(), event.msg.as_str());
        match self.producer.send(&record) {
            Ok(()) => Ok(()),
            Err(_) => Err(())
        }
    }
}
