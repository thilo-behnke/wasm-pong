
pub mod event {
    pub struct Event {
        pub msg: String
    }

    pub trait EventWriterImpl {
        fn write(&self, event: Event) -> std::io::Result<()>;
    }

    pub struct FileEventWriterImpl {}
    impl EventWriterImpl for FileEventWriterImpl {
        fn write(&self, event: Event) -> std::io::Result<()> {
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

        pub fn file() -> EventWriter {
            EventWriter {
                writer_impl: Box::new(FileEventWriterImpl {})
            }
        }
        // TODO: Kafka

        pub fn write(&self, event: Event) -> std::io::Result<()>  {
           self.writer_impl.write(event)
        }
    }
}
