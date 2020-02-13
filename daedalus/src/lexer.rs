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

#[derive(Debug, Clone, Copy, Fail)]
/// Invalid hex number is in the source.
pub struct BadHex {
    /// Place where the error occured.
    pub location: Location,
}

impl fmt::Display for BadHex {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{}: Hex number must end in \"h\" or \"H\"",
            self.location
        )
    }
}

#[derive(Debug, Clone, Copy, Fail)]
/// Invalid decimal number is in the source.
pub struct BadDecimal {
    /// Place where the error occured.
    pub location: Location,
}

impl fmt::Display for BadDecimal {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{}: Decimal number must not end in \"h\" or \"H\"",
            self.location
        )
    }
}

#[derive(Debug, Clone, Copy, Fail)]
/// Invalid number (too big) is in the source.
pub struct NumberTooBig {
    /// Place where the error occured.
    pub location: Location,
}

impl fmt::Display for NumberTooBig {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}: Number is too big", self.location)
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
    /// This token is a number literal.
    Number(u16),
    /// Just a ':' (colon).
    Colon,
    /// A regular comma ','.
    Comma,
    /// Plus operator '+'.
    Plus,
    /// Minus operator '-'.
    Minus,
    /// Multiplication operator '*'.
    Mult,
    /// Division operator '/'.
    Division,
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

impl<'buf> Lexer<'buf> {
    /// Handles the case of an error. Advances the cursor after the error.
    fn handle_error<T>(&mut self, position: SrcPosition<'buf>) -> Fallible<T> {
        self.src_iter.next();
        Err(BadChar {
            ch: position.ch,
            location: position.location,
        })?
    }

    /// Handles the case of an incoming whitespace token.
    fn handle_whitespace(&mut self, position: SrcPosition<'buf>) -> Fallible<Token<'buf>> {
        self.src_iter.next();
        while let Some(_) = self.src_iter.peek().filter(|pos| is_whitespace(pos.ch)) {
            self.src_iter.next();
        }
        Ok(Token {
            kind: TokenKind::Whitespace,
            location: position.location,
        })
    }

    /// Handles the case of an incoming newline token.
    fn handle_newline(&mut self, position: SrcPosition<'buf>) -> Fallible<Token<'buf>> {
        self.src_iter.next();
        Ok(Token {
            kind: TokenKind::Newline,
            location: position.location,
        })
    }

    /// Handles the case of an incoming identifier token.
    fn handle_ident(&mut self, position: SrcPosition<'buf>) -> Fallible<Token<'buf>> {
        let mut count = 1;

        self.src_iter.next();
        while let Some(_) = self.src_iter.peek().filter(|pos| is_ident_part(pos.ch)) {
            self.src_iter.next();
            count += 1;
        }

        Ok(Token {
            kind: TokenKind::Ident(&position.buffer[..count]),
            location: position.location,
        })
    }

    /// Handles the case of an incoming identifier token.
    fn handle_number(&mut self, position: SrcPosition<'buf>) -> Fallible<Token<'buf>> {
        let mut count = 1;

        let mut ch = position.ch;
        let mut hex = false;
        self.src_iter.next();
        while let Some(&new_pos) = self.src_iter.peek().filter(|_| is_number_part(ch)) {
            self.src_iter.next();
            ch = new_pos.ch;
            if is_hex_letter_digit(ch) {
                hex = true;
            }
            count += 1;
        }

        let num = if hex {
            if let None = self.src_iter.peek().filter(|pos| is_hex_number_end(pos.ch)) {
                Err(BadHex {
                    location: position.location,
                })?;
            }
            self.src_iter.next();

            let mut num = 0u16;
            for &ch in &position.buffer[..count] {
                num = match num
                    .checked_mul(16)
                    .and_then(|num| num.checked_add(read_hex_digit(ch) as u16))
                {
                    Some(val) => val,
                    None => Err(NumberTooBig {
                        location: position.location,
                    })?,
                };
            }
            num
        } else {
            if let None = self
                .src_iter
                .peek()
                .filter(|pos| !is_hex_number_end(pos.ch))
            {
                Err(BadDecimal {
                    location: position.location,
                })?;
            }

            let mut num = 0u16;
            for &ch in &position.buffer[..count] {
                num = match num
                    .checked_mul(16)
                    .and_then(|num| num.checked_add(read_dec_digit(ch) as u16))
                {
                    Some(val) => val,
                    None => Err(NumberTooBig {
                        location: position.location,
                    })?,
                };
            }
            num
        };

        Ok(Token {
            kind: TokenKind::Number(num),
            location: position.location,
        })
    }

    /// Handles the case of when a colon is found.
    fn handle_colon(&mut self, position: SrcPosition) -> Fallible<Token<'buf>> {
        self.src_iter.next();
        Ok(Token {
            kind: TokenKind::Colon,
            location: position.location,
        })
    }

    /// Handles the case of when a comma is found.
    fn handle_comma(&mut self, position: SrcPosition) -> Fallible<Token<'buf>> {
        self.src_iter.next();
        Ok(Token {
            kind: TokenKind::Comma,
            location: position.location,
        })
    }

    /// Handles the case of when a plus is found.
    fn handle_plus(&mut self, position: SrcPosition) -> Fallible<Token<'buf>> {
        self.src_iter.next();
        Ok(Token {
            kind: TokenKind::Plus,
            location: position.location,
        })
    }

    /// Handles the case of when a minus is found.
    fn handle_minus(&mut self, position: SrcPosition) -> Fallible<Token<'buf>> {
        self.src_iter.next();
        Ok(Token {
            kind: TokenKind::Minus,
            location: position.location,
        })
    }

    /// Handles the case of when a mult is found.
    fn handle_mult(&mut self, position: SrcPosition) -> Fallible<Token<'buf>> {
        self.src_iter.next();
        Ok(Token {
            kind: TokenKind::Mult,
            location: position.location,
        })
    }

    /// Handles the case of when a division is found.
    fn handle_division(&mut self, position: SrcPosition) -> Fallible<Token<'buf>> {
        self.src_iter.next();
        Ok(Token {
            kind: TokenKind::Division,
            location: position.location,
        })
    }

    /// Skips a comment.
    fn skip_comment(&mut self) -> Fallible<()> {
        self.src_iter.next();
        while let Some(_) = self.src_iter.peek().filter(|pos| !is_newline(pos.ch)) {
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
        } else if is_number_start(position.ch) {
            self.handle_number(position)
        } else if is_colon(position.ch) {
            self.handle_colon(position)
        } else if is_comma(position.ch) {
            self.handle_comma(position)
        } else if is_plus(position.ch) {
            self.handle_plus(position)
        } else if is_minus(position.ch) {
            self.handle_minus(position)
        } else if is_mult(position.ch) {
            self.handle_mult(position)
        } else if is_division(position.ch) {
            self.handle_division(position)
        } else {
            self.handle_error(position)
        })
    }
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

/// Tests if the byte is a character that starts a number.
fn is_number_start(ch: u8) -> bool {
    ch.is_ascii_digit()
}

/// Tests if the byte is a character that is part of a number.
fn is_number_part(ch: u8) -> bool {
    ch.is_ascii_hexdigit()
}

/// Tests if the byte is a character that is end of a hex number.
fn is_hex_number_end(ch: u8) -> bool {
    ch.eq_ignore_ascii_case(&b'h')
}

/// Tests if the byte is a character that is a hex-exclusive digit.
fn is_hex_letter_digit(ch: u8) -> bool {
    ch.is_ascii_hexdigit() && !ch.is_ascii_digit()
}

/// Tests if the byte is a colon.
fn is_colon(ch: u8) -> bool {
    ch == b':'
}

/// Tests if the byte is a colon.
fn is_comma(ch: u8) -> bool {
    ch == b','
}

/// Tests if the byte is a plus operator.
fn is_plus(ch: u8) -> bool {
    ch == b'+'
}

/// Tests if the byte is a minus operator.
fn is_minus(ch: u8) -> bool {
    ch == b'-'
}

/// Tests if the byte is a mult operator.
fn is_mult(ch: u8) -> bool {
    ch == b'*'
}

/// Tests if the byte is a division operator.
fn is_division(ch: u8) -> bool {
    ch == b'/'
}

/// Converts hex digit to plain number.
fn read_hex_digit(ch: u8) -> u8 {
    if ch.is_ascii_digit() {
        ch - b'0'
    } else if ch.is_ascii_lowercase() {
        ch - b'a' + 10
    } else {
        ch - b'A' + 10
    }
}

/// Converts decimal digit to plain number.
fn read_dec_digit(ch: u8) -> u8 {
    ch - b'0'
}
