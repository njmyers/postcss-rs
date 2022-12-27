use std::cell::RefCell;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Line {
    index: usize,
}

impl Line {
    pub fn new(index: usize) -> Self {
        return Self { index };
    }
}

impl Default for Line {
    fn default() -> Self {
        return Self::new(0);
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Lines {
    lines: RefCell<Vec<Line>>,
}

impl Lines {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn increment(&self, index: usize) {
        let mut lines = self.lines.borrow_mut();
        lines.push(Line::new(index));
    }

    pub fn value(&self) -> usize {
        return self.lines.borrow().iter().count() + 1;
    }

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
