pub mod event {
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;
    use std::fs::OpenOptions;
    use std::io::Write;
    use serde::de::DeserializeOwned;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Event {
        pub topic: String,
        pub key: Option<String>,
        pub payload: Box<dyn EventPayload>,
    }

    pub trait EventPayload : Debug + DeserializeOwned + Serialize {}

    pub trait EventWriterImpl: Send + Sync {
        fn write(&mut self, events: Vec<Event>) -> Result<(), String>;
    }

    pub struct FileEventWriterImpl {}
    impl EventWriterImpl for FileEventWriterImpl {
        fn write(&mut self, events: Vec<Event>) -> Result<(), String> {
            let event_buffer = events.iter().fold(vec![], |mut acc, e| {
                let serialized_msg = serde_json::to_string(&e.payload);
                if let Err(e) = serialized_msg {
                    // TODO: Improve error handling.
                    return acc;
                }
                let serialized_msg = serialized_msg.unwrap();
                let bytes = serialized_msg.into_bytes();
                acc.push(bytes);
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
    impl EventWriterImpl for NoopEventWriterImpl {
        fn write(&mut self, _events: Vec<Event>) -> Result<(), String> {
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

        pub fn write(&mut self, event: Event) -> Result<(), String> {
            self.write_all(vec![event])
        }

        pub fn write_all(&mut self, events: Vec<Event>) -> Result<(), String> {
            self.writer_impl.write(events)
        }
    }

    pub trait EventReaderImpl: Send + Sync {
        fn read(&mut self) -> Result<Vec<Event>, String>;
    }

    pub struct EventReader {
        reader_impl: Box<dyn EventReaderImpl>,
    }

    impl EventReader {
        pub fn new(reader_impl: Box<dyn EventReaderImpl>) -> EventReader {
            EventReader { reader_impl }
        }

        pub fn read(&mut self) -> Result<Vec<Event>, String> {
            self.reader_impl.read()
        }
    }
}
