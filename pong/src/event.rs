pub mod event {
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;
    use std::fs::OpenOptions;
    use std::io::Write;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct EventWrapper {
        pub topic: String,
        pub key: Option<String>,
        pub event: String,
    }

    #[async_trait]
    pub trait EventWriterImpl : Send {
        async fn write(&mut self, events: Vec<EventWrapper>) -> Result<(), String>;
    }

    pub struct FileEventWriterImpl {}

    #[async_trait]
    impl EventWriterImpl for FileEventWriterImpl {
        async fn write(&mut self, events: Vec<EventWrapper>) -> Result<(), String> {
            let event_buffer = events.iter().fold(vec![], |mut acc, e| {
                acc.push(e.event.as_bytes());
                acc
            }).concat();
            let options = OpenOptions::new()
                .read(true)
                .create(true)
                .write(true)
                .open("events.log");
            if let Err(e) = options {
                return Err(format!("{}", e));
            }
            let mut file = options.unwrap();
            match file.write(&*event_buffer) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{}", e)),
            }
        }
    }

    pub struct NoopEventWriterImpl {}

    #[async_trait]
    impl EventWriterImpl for NoopEventWriterImpl {
        async fn write(&mut self, _events: Vec<EventWrapper>) -> Result<(), String> {
            Ok(())
        }
    }

    pub struct EventWriter {
        writer_impl: Box<dyn EventWriterImpl>,
    }

    impl EventWriter {
        pub fn new(writer_impl: Box<dyn EventWriterImpl>) -> EventWriter {
            EventWriter { writer_impl }
        }

        pub fn noop() -> EventWriter {
            EventWriter {
                writer_impl: Box::new(NoopEventWriterImpl {}),
            }
        }

        pub fn file() -> EventWriter {
            EventWriter {
                writer_impl: Box::new(FileEventWriterImpl {}),
            }
        }

        pub async fn write(&mut self, event: EventWrapper) -> Result<(), String> {
            self.write_all(vec![event]).await
        }

        pub async fn write_all(&mut self, events: Vec<EventWrapper>) -> Result<(), String> {
            self.writer_impl.write(events).await
        }
    }

    #[async_trait]
    pub trait EventReaderImpl : Send {
        async fn read(&mut self) -> Result<Vec<EventWrapper>, String>;
    }

    pub struct EventReader {
        reader_impl: Box<dyn EventReaderImpl>,
    }

    impl EventReader {
        pub fn new(reader_impl: Box<dyn EventReaderImpl>) -> EventReader {
            EventReader { reader_impl }
        }

        pub async fn read(&mut self) -> Result<Vec<EventWrapper>, String> {
            self.reader_impl.read().await
        }
    }
}
