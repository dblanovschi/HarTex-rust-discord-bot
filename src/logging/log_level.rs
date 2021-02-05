use std::fmt::{
    Display,
    Formatter,
    Result
};

crate enum LogLevel {
    Information,
    Debug,
    Warning,
    Error,
    Verbose
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match *self {
            LogLevel::Information => write!(f, "INFO    "),
            LogLevel::Debug => write!(f, "DEBUG   "),
            LogLevel::Warning => write!(f, "WARNING "),
            LogLevel::Error => write!(f, "ERROR   "),
            LogLevel::Verbose => write!(f, "VERBOSE ")
        }
    }
}
