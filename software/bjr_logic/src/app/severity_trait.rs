pub enum ErrorSeverity {
    Panic,
    ImmediateShutdown,
    GracefulShutdown,
    Report,
    Ignore,
}
pub trait Severity {
    fn get_severity(&self) -> ErrorSeverity;
}
