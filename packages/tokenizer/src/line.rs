use std::cell::RefCell;

/// Records the byte index of a single newline character.
///
/// Used by [`Lines`] to reconstruct line/column information from a flat
/// character index.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Line {
    index: usize,
}

impl Line {
    /// Creates a new `Line` whose newline is at `index`.
    pub fn new(index: usize) -> Self {
        return Self { index };
    }
}

impl Default for Line {
    fn default() -> Self {
        return Self::new(0);
    }
}

/// Tracks all newline positions encountered during tokenization.
///
/// As the tokenizer advances through the source, it calls [`Lines::increment`]
/// each time it encounters a newline.  The resulting collection can then
/// translate any flat byte index back into a 1-based line number
/// ([`Lines::value`]) or a 1-based column offset ([`Lines::offset`]).
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Lines {
    lines: RefCell<Vec<Line>>,
}

impl Lines {
    /// Creates an empty `Lines` tracker (no newlines seen yet).
    pub fn new() -> Self {
        return Self::default();
    }

    /// Records a newline at the given byte `index`.
    pub fn increment(&self, index: usize) {
        let mut lines = self.lines.borrow_mut();
        lines.push(Line::new(index));
    }

    /// Returns the current 1-based line number.
    ///
    /// This equals the number of newlines recorded so far plus one.
    pub fn value(&self) -> usize {
        return self.lines.borrow().len() + 1;
    }

    /// Returns the number of newlines recorded so far.
    pub fn len(&self) -> usize {
        self.lines.borrow().len()
    }

    /// Returns `true` if no newlines have been recorded yet.
    pub fn is_empty(&self) -> bool {
        self.lines.borrow().is_empty()
    }

    /// Truncates the newline records to `len` entries.
    ///
    /// Used by [`Tokenizer::rewind`] to restore line-tracking state.
    pub fn truncate(&self, len: usize) {
        self.lines.borrow_mut().truncate(len);
    }

    /// Returns the 1-based column offset for a given byte `index`.
    ///
    /// If at least one newline has been recorded, the column is computed as
    /// `index - last_newline_index`.  For the first line (no newlines yet) the
    /// column is `index + 1`.
    pub fn offset(&self, index: usize) -> usize {
        if let Some(line) = self.lines.borrow().last() {
            return index - line.index;
        }

        return index + 1;
    }
}

impl Default for Lines {
    fn default() -> Self {
        return Self {
            lines: RefCell::new(Vec::new()),
        };
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
        let lines = Lines::new();
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
        let lines = Lines::new();
        // Simulate a newline at index 5 (e.g., the '\n' character)
        lines.increment(5);
        // A character at index 6 is the first character of line 2 → column 1
        assert_eq!(lines.offset(6), 1);
        // A character at index 8 is the third character of line 2 → column 3
        assert_eq!(lines.offset(8), 3);
    }

    #[test]
    fn test_line_default() {
        let line = Line::default();
        assert_eq!(line.index, 0);
    }
}
