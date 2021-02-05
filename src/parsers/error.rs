use std::{
    error::Error,
    fmt::{
        Display,
        Formatter,
        Result as FmtResult
    }
};

#[derive(Debug)]
crate struct ParseError(crate String);

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl Error for ParseError { }
