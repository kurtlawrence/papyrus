use super::*;
use cmdtree::Commander;

pub struct TreeCompleter {
    items: Vec<String>,
}

impl TreeCompleter {
    pub fn build<T>(cmdr: &Commander<T>) -> Self {
        let prefix = if cmdr.at_root() { "." } else { "" };

        let mut items = cmdtree::completion::create_tree_completion_items(&cmdr);

        items.iter_mut().for_each(|x| {
            if cmdr.at_root() {
                x.insert(0, '.');
            }
        });

        Self { items }
    }
}

impl<T: Terminal> Completer<T> for TreeCompleter {
    fn complete(
        &self,
        _word: &str,
        prompter: &Prompter<T>,
        _start: usize,
        _end: usize,
    ) -> Option<Vec<Completion>> {
        let line = prompter.buffer();

        let v: Vec<_> = cmdtree::completion::tree_completions(line, self.items.iter())
            .map(|x| Completion::simple(x.to_string()))
            .collect();

        if v.len() > 0 {
            Some(v)
        } else {
            None
        }
    }
}

pub struct ActionArgComplete {
    pub items: Vec<cmdtree::completion::ActionMatch>,
}

impl ActionArgComplete {
    pub fn build<T>(cmdr: &Commander<T>) -> Self {

		let mut items = cmdtree::completion::create_action_completion_items(&cmdr);

		items.iter_mut().for_each(|x| {
            if cmdr.at_root() {
                x.match_str.insert(0, '.');
            }
        });

        Self { items }
    }

	pub fn find<'a>(&self, line: &'a str, valid: &[&str]) -> Option<ArgComplete<'a>> {
		self.items.iter().find(|x| line.starts_with(x.match_str.as_str())).and_then(|x| if valid.contains(&x.qualified_path.as_str()) {
			let word_start = linefeed::complete::word_break_start(line, " ");
			Some(ArgComplete{
				line: &line[x.match_str.len()..],
				word: &line[word_start..],
				word_start,
			})
		} else {
			None
		})
	}
}

pub struct ArgComplete<'a> {
    pub line: &'a str,    
	pub word: &'a str,
    pub word_start: usize,
}
