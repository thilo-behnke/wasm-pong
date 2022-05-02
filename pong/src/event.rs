
pub mod event {
    use std::fs::OpenOptions;
    use std::io::Write;

    pub struct Event {
        pub topic: String,
        pub key: String,
        pub msg: String
    }

    pub trait EventWriterImpl {
        fn write(&self, event: Event) -> Result<(), ()>;
    }

    pub struct FileEventWriterImpl {}
    impl EventWriterImpl for FileEventWriterImpl {
        fn write(&self, event: Event) -> Result<(), ()> {
            let options = OpenOptions::new().read(true).create(true).write(true).open("events.log");
            if let Err(_) = options {
                return Err(());
            }
            let mut file = options.unwrap();
            match file.write(event.msg.as_bytes()) {
                Ok(_) => Ok(()),
                Err(e) => Err(())
            }
        }
    }

    pub struct NoopEventWriterImpl {}
    impl EventWriterImpl for NoopEventWriterImpl {
        fn write(&self, event: Event) -> Result<(), ()> {
            todo!()
        }
    }

    pub struct EventWriter {
        writer_impl: Box<dyn EventWriterImpl>
    }

    impl EventWriter {
        fn new(writer_impl: Box<dyn EventWriterImpl>) -> EventWriter {
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
        // TODO: Kafka

        pub fn write(&self, event: Event) -> Result<(), ()> {
           self.writer_impl.write(event)
        }
    }
}
