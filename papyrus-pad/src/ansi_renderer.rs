use super::*;
use azul::css::CssProperty;

const PROPERTY_STR: &str = "ansi_esc_color";

pub struct AnsiRenderer {
    /// Draw from this to avoid constantly creating new `TextId`s
    id_pool: Vec<TextId>,
    /// Each text component to display.
    display: Vec<(TextId, CssProperty)>,
    /// The index in `display` which starts a new line.
    lines: Vec<usize>,
}

impl AnsiRenderer {
    pub fn new() -> Self {
        Self {
            id_pool: Vec::new(),
            display: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn update_text(&mut self, text: &str, app_resources: &mut AppResources) {
        self.display.clear();
        self.lines.clear();
        self.id_pool
            .drain(..)
            .for_each(|id| app_resources.delete_text(id));

        let categorised = cansi::categorise_text(text);

        let mut idx = 0;

        for line in cansi::line_iter(&categorised) {
            for cat in line {
                let id = app_resources.add_text(cat.text);

                self.id_pool.push(id);

                let prop = StyleTextColor(crate::colour::map(&cat.fg_colour)).into();

                self.display.push((id, prop));

                idx += 1;
            }

            self.lines.push(idx);
        }
    }

    pub fn dom<T>(&self) -> Dom<T> {
        // piece together the components
        let mut container = Dom::div();

        let mut start = 0;

        for &end in &self.lines {
            let mut line_div = Dom::div().with_class("repl-terminal-line");

            for (id, prop) in &self.display[start..end] {
                line_div.add_child(
                    Dom::text_id(*id)
                        .with_class("repl-terminal-text")
                        .with_css_override(PROPERTY_STR, prop.clone()),
                );
            }

            container.add_child(line_div);

            start = end;
        }

        container
    }
}
