pub mod utils {
    pub trait LoggerFactory {
        fn get(&self, name: &str) -> Box<dyn Logger>;
    }

    pub struct DefaultLoggerFactory {
        proto: Box<dyn Logger>,
    }

    impl LoggerFactory for DefaultLoggerFactory {
        fn get(&self, name: &str) -> Box<dyn Logger> {
            let mut clone = self.proto.box_clone();
            clone.set_name(name);
            clone
        }
    }

    impl DefaultLoggerFactory {
        pub fn new(proto: Box<dyn Logger>) -> Box<dyn LoggerFactory> {
            Box::new(DefaultLoggerFactory { proto })
        }
        pub fn noop() -> Box<dyn LoggerFactory> {
            Box::new(DefaultLoggerFactory {
                proto: Box::new(NoopLogger {}),
            })
        }
    }

    pub trait Logger {
        fn box_clone(&self) -> Box<dyn Logger>;
        fn set_name(&mut self, name: &str);
        fn log(&self, msg: &str);
    }

    #[derive(Clone)]
    pub struct NoopLogger {}

    impl Logger for NoopLogger {
        fn box_clone(&self) -> Box<dyn Logger> {
            Box::new(self.clone())
        }

        fn set_name(&mut self, _name: &str) {}

        fn log(&self, _msg: &str) {}
    }
}

pub mod number_utils {
    pub fn is_in_range(n: f64, from: f64, to: f64) -> bool {
        return from <= n && n <= to;
    }
}
