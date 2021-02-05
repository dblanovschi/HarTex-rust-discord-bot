use std::{
    error::Error
};

#[derive(Clone)]
crate struct CommandFailed {
    crate command: &'static str,
    crate error: String
}
