use crate::source::Location;

pub enum Operator {}

pub enum TokenKind {
    Ident(Box<[u8]>),
    Literal(u8),
    Operator(Operator),
    Colon,
}

pub struct Token {
    pub kind: TokenKind,
    pub loc: Location,
}
