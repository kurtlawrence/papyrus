use super::*;
use azul::css::CssProperty;
use papyrus::output::OutputChange;

const PROPERTY_STR: &str = "ansi_esc_color";

pub struct AnsiLineRenderer {
    id: Option<TextId>,
    /// Each text component to display.
    display: Vec<(CharRange, Option<CssProperty>)>,
}

impl AnsiLineRenderer {
    pub fn new() -> Self {
        Self {
            id: None,
            display: Vec::new(),
        }
    }

    pub fn update_text(&mut self, text: &str, app_resources: &mut AppResources) {
        self.display.clear();

        let words = azul::text_layout::split_text_into_words(text);

        self.id = Some(app_resources.add_text(words));

        let words = app_resources.get_text(self.id.as_ref().unwrap()).unwrap();

        let categorised = cansi::categorise_text(words.get_str());

        for cat in categorised {
            let col = cat.fg_colour;
            use cansi::Color::*;

            let prop = if col == Black || col == White || col == BrightBlack || col == BrightWhite {
                None
            } else {
                Some(StyleTextColor(crate::colour::map(&cat.fg_colour)).into())
            };

            let chrange = words
                .convert_byte_range(cat.start..cat.end)
                .expect("should be on char boundaries");

            self.display.push((chrange, prop));
        }
    }

    pub fn dom<T>(&self) -> Dom<T> {
        let mut line = Dom::div().with_class("ansi-renderer-line");

        if let Some(id) = self.id {
            for (chrange, prop) in &self.display {
                let mut child = Dom::text_slice(id, *chrange).with_class("ansi-renderer-text");

                if let Some(prop) = prop {
                    child.add_css_override(PROPERTY_STR, prop.clone());
                }

                line.add_child(child);
            }
        }

        line
    }
}

pub struct ReplOutputRenderer {
    pub lines: Vec<AnsiLineRenderer>,
    pub rx: papyrus::output::Receiver,
}

impl ReplOutputRenderer {
    pub fn new(receiver: papyrus::output::Receiver) -> Self {
        Self {
            lines: vec![AnsiLineRenderer::new()],
            rx: receiver,
        }
    }

    pub fn handle_line_changes(&mut self, app_resources: &mut AppResources) -> bool {
        let mut msgs = false;

        for chg in self.rx.try_iter() {
            msgs = true;

            match chg {
                OutputChange::CurrentLine(line) => {
                    self.lines
                        .last_mut()
                        .map(|x| x.update_text(&line, app_resources));
                }
                OutputChange::NewLine => self.lines.push(AnsiLineRenderer::new()),
            }
        }

        msgs
    }

    pub fn dom<T>(&self) -> Dom<T> {
        self.lines
            .iter()
            .map(|x| x.dom())
            .collect::<Dom<T>>()
            .with_class("repl-output-renderer")
    }
}
