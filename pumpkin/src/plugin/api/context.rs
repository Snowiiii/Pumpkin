pub trait PluginContext: Send + Sync {
    fn get_logger(&self) -> Box<dyn Logger>;
}

pub trait Logger {
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
    fn error(&self, message: &str);
}
