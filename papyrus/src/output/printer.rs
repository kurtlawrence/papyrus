use super::*;

pub trait Printer {
    fn alter_line(&mut self, line_index: usize, line: &str);
}
