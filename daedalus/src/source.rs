use std::{fmt, slice};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A location in the source code, specially an error.
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

#[derive(Debug, Clone)]
pub struct SrcIter<'buf> {
    inner: slice::Iter<'buf, u8>,
    curr_loc: Location,
}

impl<'buf> SrcIter<'buf> {
    pub fn new(src: &'buf [u8]) -> Self {
        Self { inner: src.iter(), curr_loc: Location::default() }
    }
}

impl<'buf> Iterator for SrcIter<'buf> {
    type Item = (u8, Location);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|&byte| {
            let loc = self.curr_loc;

            if byte == b'\n' {
                self.curr_loc.line += 1;
                self.curr_loc.column = 1;
            } else {
                self.curr_loc.column += 1;
            }

            (byte, loc)
        })
    }
}
