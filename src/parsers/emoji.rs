use super::{
    error::ParseError,
    Parser,
    ParseResult
};

use regex::Regex;

crate struct EmojiParser;

#[derive(Clone, Debug)]
crate struct Emoji {
    crate name: String,
    crate id: u64,
    crate animated: bool
}

impl EmojiParser {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for EmojiParser {
    fn default() -> Self {
        Self
    }
}

impl Parser for EmojiParser {
    type Output = Emoji;

    fn parse(&self, input: String) -> ParseResult<Self::Output> {
        let regex = Regex::new(r#"<(a?):(\w+):(\d+)>"#).unwrap();

        if let Some(captures) = regex.captures(&input) {
            let mut is_animated = false;
            let mut name = String::new();
            let mut emoji_id = 0u64;

            if let Some(c) = captures.get(1) {
                match c.as_str() {
                    "a" => {
                        is_animated = true;

                        if let Some(n) = captures.get(2) {
                            name = String::from(n.as_str());
                        }
                        else {
                            return Err(ParseError("Could not parse emoji name.".to_string()));
                        }

                        if let Some(id) = captures.get(3) {
                            if let Ok(id_num) = id.as_str().parse() {
                                emoji_id = id_num;
                            }
                            else if let Err(error) = id.as_str().parse::<u64>() {
                                return Err(ParseError(format!("Could not parse emoji id: {}", error)));
                            }
                            else {
                                unreachable!()
                            }
                        }
                    },
                    _ => {
                        is_animated = false;

                        if let Some(n) = captures.get(2) {
                            name = String::from(n.as_str());
                        }
                        else {
                            return Err(ParseError("Could not parse emoji name.".to_string()));
                        }

                        if let Some(id) = captures.get(3) {
                            if let Ok(id_num) = id.as_str().parse() {
                                emoji_id = id_num;
                            }
                            else if let Err(error) = id.as_str().parse::<u64>() {
                                return Err(ParseError(format!("Could not parse emoji id: {}", error)));
                            }
                            else {
                                unreachable!()
                            }
                        }
                    }
                }
            }

            Ok(Emoji {
                name,
                id: emoji_id,
                animated: is_animated
            })
        }
        else {
            Err(ParseError("Could not parse provided string.".to_string()))
        }
    }
}
