use std::{fmt, slice};

/// A location in the source code, specially an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    /// Referred line.
    pub line: usize,
    /// Referred column.
    pub column: usize,
}

impl Default for Location {
    fn default() -> Self {
        Self { line: 1, column: 1 }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "in line {}, column {}", self.line, self.column)
    }
}

/// Iterator over source byte-characters.
#[derive(Debug, Clone)]
pub struct SrcIter<'buf> {
    /// Iterator over elements of the slice.
    inner: slice::Iter<'buf, u8>,
    /// Location of the current character.
    curr_loc: Location,
}

impl<'buf> SrcIter<'buf> {
    /// Creates a new iterator from the given source.
    pub fn new(src: &'buf [u8]) -> Self {
        Self { inner: src.iter(), curr_loc: Location::default() }
    }
}

/// A position in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SrcPosition<'buf> {
    /// Character in this position.
    pub ch: u8,
    /// Buffer from the current position until the end.
    pub buffer: &'buf [u8],
    /// Location of this position in line and column number.
    pub location: Location,
}

impl<'buf> Iterator for SrcIter<'buf> {
    type Item = SrcPosition<'buf>;

    fn next(&mut self) -> Option<Self::Item> {
        let buffer = self.inner.as_slice();
        let &ch = self.inner.next()?;
        let location = self.curr_loc;

        if ch == b'\n' {
            self.curr_loc.line += 1;
            self.curr_loc.column = 1;
        } else {
            self.curr_loc.column += 1;
        }

        Some(SrcPosition { ch, buffer, location })
    }
}
