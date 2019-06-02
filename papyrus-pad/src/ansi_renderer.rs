use super::*;
use azul::css::CssProperty;

const PROPERTY_STR: &str = "ansi_esc_color";

pub struct AnsiRenderer {
    id: Option<TextId>,
    /// Each text component to display.
    display: Vec<(CharRange, CssProperty)>,
    /// The index in `display` which starts a new line.
    lines: Vec<usize>,
}

impl AnsiRenderer {
    pub fn new() -> Self {
        Self {
            id: None,
            display: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn update_text(&mut self, text: &str, app_resources: &mut AppResources) {
        self.display.clear();
        self.lines.clear();

        self.id = Some(app_resources.add_text(text));

        let words = app_resources.get_text(self.id.as_ref().unwrap()).unwrap();

        let categorised = cansi::categorise_text(words.get_str());

        let mut idx = 0;

        for line in cansi::line_iter(&categorised) {
            for cat in line {
                let prop = StyleTextColor(crate::colour::map(&cat.fg_colour)).into();

                let chrange = words
                    .convert_byte_range(cat.start..cat.end)
                    .expect("should be on char boundaries");

                self.display.push((chrange, prop));

                idx += 1;
            }

            self.lines.push(idx);
        }
    }

    pub fn dom<T>(&self) -> Dom<T> {
        // piece together the components
        let mut container = Dom::div();

        let mut start = 0;

        if let Some(id) = self.id {
            for &end in &self.lines {
                let mut line_div = Dom::div().with_class("ansi-renderer-line");

                for (chrange, prop) in &self.display[start..end] {
                    line_div.add_child(
                        Dom::text_slice(id, *chrange)
                            .with_class("ansi-renderer-text")
                            .with_css_override(PROPERTY_STR, prop.clone()),
                    );
                }

                container.add_child(line_div);

                start = end;
            }
        }

        container
    }
}
