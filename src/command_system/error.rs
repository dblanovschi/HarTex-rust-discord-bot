use std::{
    error::Error,
    fmt::{
        Debug as StdDebug,
        Display,
        Formatter,
        Result
    }
};

crate struct CommandError(crate String);

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl StdDebug for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Command Error: {}", self.0)
    }
}

impl Error for CommandError {}
