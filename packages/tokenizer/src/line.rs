/// Tracks all newline positions encountered during tokenization.
///
/// As the tokenizer advances through the source, it calls [`Lines::increment`]
/// each time it encounters a newline. The resulting collection can then
/// translate any flat byte index back into a 1-based line number
/// ([`Lines::value`]) or a 1-based column offset ([`Lines::offset`]).
#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct Lines {
    /// Byte index of each newline character seen so far.
    lines: Vec<usize>,
}

impl Lines {
    /// Creates an empty `Lines` tracker (no newlines seen yet).
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a newline at the given byte `index`.
    #[inline]
    pub fn increment(&mut self, index: usize) {
        self.lines.push(index);
    }

    /// Returns the current 1-based line number.
    ///
    /// This equals the number of newlines recorded so far plus one.
    #[inline]
    pub fn value(&self) -> usize {
        self.lines.len() + 1
    }

    /// Returns the number of newlines recorded so far.
    #[inline]
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Truncates the newline records to `len` entries.
    ///
    /// Used by [`Tokenizer::rewind`] to restore line-tracking state.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.lines.truncate(len);
    }

    /// Returns the 1-based column offset for a given byte `index`.
    ///
    /// If at least one newline has been recorded, the column is computed as
    /// `index - last_newline_index`.  For the first line (no newlines yet) the
    /// column is `index + 1`.
    #[inline]
    pub fn offset(&self, index: usize) -> usize {
        match self.lines.last() {
            Some(&last) => index - last,
            None => index + 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_line_is_one() {
        let lines = Lines::new();
        assert_eq!(lines.value(), 1);
    }

    #[test]
    fn test_increment_increases_line_count() {
        let mut lines = Lines::new();
        lines.increment(5);
        assert_eq!(lines.value(), 2);
        lines.increment(10);
        assert_eq!(lines.value(), 3);
    }

    #[test]
    fn test_offset_first_line() {
        let lines = Lines::new();
        // On the first line there are no recorded newlines, so offset = index + 1
        assert_eq!(lines.offset(0), 1);
        assert_eq!(lines.offset(4), 5);
    }

    #[test]
    fn test_offset_after_newline() {
        let mut lines = Lines::new();
        // Simulate a newline at index 5 (e.g., the '\n' character)
        lines.increment(5);
        // A character at index 6 is the first character of line 2 → column 1
        assert_eq!(lines.offset(6), 1);
        // A character at index 8 is the third character of line 2 → column 3
        assert_eq!(lines.offset(8), 3);
    }
}
