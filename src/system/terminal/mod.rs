use std::{
    fmt::{
        Display,
        Formatter,
        Result
    }
};

crate struct Ansi256 {
    pub colour: u16
}

impl Ansi256 {
    crate fn reset() -> String {
        "\x1B[0m".to_string()
    }
}

impl Display for Ansi256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        return write!(f, "\x1B[38;5;{}m", self.colour);
    }
}
