use crate::source::{Location, SrcIter};
use error::Fallible;
use failure::Fail;
use std::{fmt, iter::Peekable};

#[derive(Debug, Clone, Copy, Fail)]
/// Invalid char is in the source code.
pub struct BadChar {
    /// The caracter found.
    pub ch: u8,
    /// Place where the error occured.
    pub location: Location,
}

impl fmt::Display for BadChar {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}: ", self.location)?;
        if self.ch.is_ascii_graphic() {
            write!(fmt, "Invalid character \"{}\"", self.ch as char)
        } else {
            write!(fmt, "Invalid character {:x}h", self.ch)
        }
    }
}

#[derive(Debug, Clone)]
pub enum TokenKind {
    Whitespace,
    Newline,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub location: Location,
}

#[derive(Debug, Clone)]
pub struct Lexer<'buf> {
    src_iter: Peekable<SrcIter<'buf>>,
}

fn is_whitespace(ch: u8) -> bool {
    match ch {
        b' ' | b'\t' | b'\r' => true,
        _ => false,
    }
}

fn is_newline(ch: u8) -> bool {
    ch == b'\n'
}

fn is_comment_start(ch: u8) -> bool {
    ch == b';'
}

impl<'buf> Lexer<'buf> {
    fn handle_error<T>(&mut self, ch: u8, location: Location) -> Fallible<T> {
        self.src_iter.next();
        Err(BadChar { ch, location })?
    }

    fn handle_whitespace(&mut self, location: Location) -> Fallible<Token> {
        self.src_iter.next();
        while self.src_iter.peek().map_or(false, |&(ch, _)| is_whitespace(ch)) {
            self.src_iter.next();
        }
        Ok(Token { kind: TokenKind::Whitespace, location })
    }

    fn handle_newline(&mut self, location: Location) -> Fallible<Token> {
        self.src_iter.next();
        Ok(Token { kind: TokenKind::Newline, location })
    }

    fn skip_comment(&mut self) -> Fallible<()> {
        self.src_iter.next();
        while self.src_iter.peek().map_or(false, |&(ch, _)| !is_newline(ch)) {
            self.src_iter.next();
        }
        Ok(())
    }
}

impl<'buf> Iterator for Lexer<'buf> {
    type Item = Fallible<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        let &(mut ch, mut location) = self.src_iter.peek()?;

        while is_comment_start(ch) {
            if let Err(e) = self.skip_comment() {
                return Some(Err(e));
            }
            let &(new_ch, new_location) = self.src_iter.peek()?;
            ch = new_ch;
            location = new_location;
        }

        Some(if is_whitespace(ch) {
            self.handle_whitespace(location)
        } else if is_newline(ch) {
            self.handle_newline(location)
        } else {
            self.handle_error(ch, location)
        })
    }
}
