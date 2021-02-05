mod emoji;
mod error;

crate use emoji::EmojiParser;

crate type ParseResult<T> = Result<T, error::ParseError>;

crate trait Parser {
    type Output;
    type State = ();

    fn parse(&self, input: String) -> ParseResult<Self::Output>;
}
