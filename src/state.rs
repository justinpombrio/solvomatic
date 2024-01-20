use std::fmt;
use std::fmt::Display;
use std::hash::Hash;

const TEXT_BOX_PADDING: usize = 4;
const TEXT_BOX_WIDTH: usize = 90;

/// The state for some kind of puzzle. It maps `Var` to `Value`, and can be nicely `Display`ed.
/// Importantly, not all `Var`s will have a `Value`. The default state should have `None` for all
/// `Var`s.
pub trait State: Display + 'static {
    type Var: fmt::Debug + Hash + Eq + Ord + Clone + Send + Sync + 'static;
    type Value: fmt::Debug + Hash + Eq + Ord + Clone + Send + Sync + 'static;
    type MetaData: Clone + Send + Sync;

    fn new(metadata: &Self::MetaData) -> Self;

    fn set(&mut self, var: Self::Var, val: Self::Value);
}

/// A bunch of states. This type exists solely for its `Display` method, which will show all of its
/// states, and will print put them side by side when they fit.
pub struct StateSet<S: State>(pub Vec<S>);

impl<S: State> Display for StateSet<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut text_box = TextBox::new(TEXT_BOX_WIDTH);
        for state in &self.0 {
            text_box.append(format!("{}", state));
        }
        for line in text_box.completed_lines {
            writeln!(f, "{}", line)?;
        }
        for line in text_box.cur_lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

struct TextBox {
    max_width: usize,
    completed_lines: Vec<String>,
    cur_width: usize,
    cur_lines: Vec<String>,
}

impl TextBox {
    fn new(max_width: usize) -> TextBox {
        TextBox {
            max_width,
            completed_lines: Vec::new(),
            cur_width: 0,
            cur_lines: Vec::new(),
        }
    }

    fn print_line(&mut self, row: usize, col: usize, line: &str) {
        while row >= self.cur_lines.len() {
            self.cur_lines.push(String::new());
        }
        let cur_line = self.cur_lines.get_mut(row).unwrap();
        let len = cur_line.chars().count();
        if col > len {
            let missing_len = col - len;
            cur_line.push_str(&format!("{:spaces$}", "", spaces = missing_len));
        }
        cur_line.push_str(line);
        //self.cur_width = self.cur_width.max(cur_line.chars().count());
    }

    fn append(&mut self, state: String) {
        let mut state_width = 0;
        for line in state.lines() {
            state_width = state_width.max(line.chars().count());
        }
        state_width += TEXT_BOX_PADDING;

        if self.cur_width == 0 || self.cur_width + state_width <= self.max_width {
            // It fits on the current lines. Print it there.
            let col = self.cur_width + TEXT_BOX_PADDING;
            for (row, line) in state.lines().enumerate() {
                self.print_line(row, col, line);
            }
            self.cur_width += state_width;
        } else {
            // It doesn't fit. Finish our current lines and start new ones.
            for line in self.cur_lines.drain(..) {
                self.completed_lines.push(line);
            }
            self.completed_lines.push(String::new());
            for state_line in state.lines() {
                let mut line = format!("{:spaces$}", "", spaces = TEXT_BOX_PADDING);
                line.push_str(state_line);
                self.cur_lines.push(line);
            }
            self.cur_width = state_width;
        }
    }
}
