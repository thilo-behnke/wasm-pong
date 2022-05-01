pub mod utils {
    pub trait Logger : Clone {
        fn log(&self, msg: &str);
    }

    #[derive(Clone)]
    pub struct NoopLogger {}
    impl Logger for NoopLogger {
        fn log(&self, msg: &str) {}
    }
}
