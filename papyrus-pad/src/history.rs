use std::collections::VecDeque;

/// Stores unique history items and the currently selected item.
/// Provides mechanisms to move forwards or backwards through the histories.
pub struct History {
    items: VecDeque<String>,
    pos: Option<usize>,
}

impl History {
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
            pos: None,
        }
    }

    pub fn add_unique(&mut self, item: String) {
        let idx = self
            .items
            .iter()
            .enumerate()
            .find(|(_, x)| x == &&item)
            .map(|(idx, _)| idx);

        if let Some(idx) = idx {
            self.items.remove(idx);
        }

        self.items.push_front(item);
    }

    pub fn reset_position(&mut self) {
        self.pos = None;
    }

    pub fn move_backwards(&mut self) -> Option<&String> {
        if self.items.is_empty() {
            None
        } else {
            let idx = self.pos.map(|x| x + 1).unwrap_or(0);
            let idx = std::cmp::min(idx, self.items.len().saturating_sub(1));
            self.pos = Some(idx);
            self.items.get(idx)
        }
    }

    pub fn move_forwards(&mut self) -> Option<&String> {
        let idx = self.pos.unwrap_or(0);
        if idx == 0 {
            self.pos = None;
            None
        } else {
            let idx = idx - 1;
            self.pos = Some(idx);
            self.items.get(idx)
        }
    }
}
