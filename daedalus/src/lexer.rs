use crate::source::{Location, SrcIter, SrcPosition};
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

/// A kind of a token.
#[derive(Debug, Clone)]
pub enum TokenKind<'buf> {
    /// This token is whitespace.
    Whitespace,
    /// This token is a newline.
    Newline,
    /// This token is an idenitifer.
    Ident(&'buf [u8]),
}

/// A token, with a given kind and location of occurence.
#[derive(Debug, Clone)]
pub struct Token<'buf> {
    /// Kind of this token.
    pub kind: TokenKind<'buf>,
    /// Location of this token's occurence.
    pub location: Location,
}

/// A lexer. Translates bytes to tokens.
#[derive(Debug, Clone)]
pub struct Lexer<'buf> {
    /// Iterator over source code bytes.
    src_iter: Peekable<SrcIter<'buf>>,
}

/// Tests if the byte is a whitespace character.
fn is_whitespace(ch: u8) -> bool {
    match ch {
        b' ' | b'\t' | b'\r' => true,
        _ => false,
    }
}

/// Tests if the byte is a newline character.
fn is_newline(ch: u8) -> bool {
    ch == b'\n'
}

/// Tests if the byte is a character that starts a comment.
fn is_comment_start(ch: u8) -> bool {
    ch == b';'
}

/// Tests if the byte is a character that starts an identifier.
fn is_ident_start(ch: u8) -> bool {
    ch.is_ascii_alphabetic() || ch == b'_'
}

/// Tests if the byte is a character that starts an identifier.
fn is_ident_part(ch: u8) -> bool {
    ch.is_ascii_alphanumeric() || ch == b'_'
}

impl<'buf> Lexer<'buf> {
    /// Handles the case of an error. Advances the cursor after the error.
    fn handle_error<T>(&mut self, position: SrcPosition<'buf>) -> Fallible<T> {
        self.src_iter.next();
        Err(BadChar { ch: position.ch, location: position.location })?
    }

    /// Handles the case of an incoming whitespace token.
    fn handle_whitespace(
        &mut self,
        position: SrcPosition<'buf>,
    ) -> Fallible<Token<'buf>> {
        self.src_iter.next();
        while self.src_iter.peek().map_or(false, |pos| is_whitespace(pos.ch)) {
            self.src_iter.next();
        }
        Ok(Token { kind: TokenKind::Whitespace, location: position.location })
    }

    /// Handles the case of an incoming newline token.
    fn handle_newline(
        &mut self,
        position: SrcPosition<'buf>,
    ) -> Fallible<Token<'buf>> {
        self.src_iter.next();
        Ok(Token { kind: TokenKind::Newline, location: position.location })
    }

    /// Handles the case of an incoming identifier token.
    fn handle_ident(
        &mut self,
        position: SrcPosition<'buf>,
    ) -> Fallible<Token<'buf>> {
        let mut count = 1;

        self.src_iter.next();
        while self.src_iter.peek().map_or(false, |pos| is_ident_part(pos.ch)) {
            self.src_iter.next();
            count += 1;
        }

        Ok(Token {
            kind: TokenKind::Ident(&position.buffer[.. count]),
            location: position.location,
        })
    }

    /// Skips a comment.
    fn skip_comment(&mut self) -> Fallible<()> {
        self.src_iter.next();
        while self.src_iter.peek().map_or(false, |pos| !is_newline(pos.ch)) {
            self.src_iter.next();
        }
        Ok(())
    }
}

impl<'buf> Iterator for Lexer<'buf> {
    type Item = Fallible<Token<'buf>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut position = *self.src_iter.peek()?;

        while is_comment_start(position.ch) {
            if let Err(e) = self.skip_comment() {
                return Some(Err(e));
            }
            position = *self.src_iter.peek()?;
        }

        Some(if is_whitespace(position.ch) {
            self.handle_whitespace(position)
        } else if is_newline(position.ch) {
            self.handle_newline(position)
        } else if is_ident_start(position.ch) {
            self.handle_ident(position)
        } else {
            self.handle_error(position)
        })
    }
}
