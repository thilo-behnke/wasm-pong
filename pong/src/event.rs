
pub mod event {
    use std::fs::OpenOptions;
    use std::io::Write;

    #[derive(Debug)]
    pub struct Event {
        pub topic: String,
        pub key: String,
        pub msg: String
    }

    pub trait EventWriterImpl : Send + Sync {
        fn write(&mut self, event: Event) -> Result<(), String>;
    }

    pub struct FileEventWriterImpl {}
    impl EventWriterImpl for FileEventWriterImpl {
        fn write(&mut self, event: Event) -> Result<(), String> {
            let options = OpenOptions::new().read(true).create(true).write(true).open("events.log");
            if let Err(e) = options {
                return Err(format!("{}", e));
            }
            let mut file = options.unwrap();
            match file.write(event.msg.as_bytes()) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{}", e))
            }
        }
    }

    pub struct NoopEventWriterImpl {}
    impl EventWriterImpl for NoopEventWriterImpl {
        fn write(&mut self, event: Event) -> Result<(), String> {
            todo!()
        }
    }

    pub struct EventWriter {
        writer_impl: Box<dyn EventWriterImpl>
    }

    impl EventWriter {
        pub fn new(writer_impl: Box<dyn EventWriterImpl>) -> EventWriter {
            EventWriter {
                writer_impl
            }
        }

        pub fn noop() -> EventWriter {
            EventWriter {
                writer_impl: Box::new(NoopEventWriterImpl {})
            }
        }

        pub fn file() -> EventWriter {
            EventWriter {
                writer_impl: Box::new(FileEventWriterImpl {})
            }
        }

        pub fn write(&mut self, event: Event) -> Result<(), String> {
           self.writer_impl.write(event)
        }
    }

    pub trait EventReaderImpl {
        fn read(&mut self) -> Vec<Event>;
    }

    pub struct EventReader {
        reader_impl: Box<dyn EventReaderImpl>
    }

    impl EventReader {
        pub fn new(reader_impl: Box<dyn EventReaderImpl>) -> EventReader {
            EventReader {
                reader_impl
            }
        }

        pub fn read(&mut self) -> Vec<Event> {
            self.reader_impl.read()
        }
    }
}
