#[macro_use]
extern crate criterion;

#[macro_use]
extern crate papyrus;

use criterion::Criterion;

use azul::prelude::*;
use colored::Colorize;
use papyrus::prelude::*;
use papyrus_pad::*;
use std::sync::{Arc, RwLock};

// Ansi rendering
// Benchmarks the parsing and the dom creation
fn ansi_rendering(c: &mut Criterion) {
    use ansi_renderer::*;

    // # categorising -- check speed
    let txt = large_colored_txt();
    c.bench_function("categorising text", move |b| {
        b.iter(|| cansi::categorise_text(&txt))
    });

    // # update_text
    let txt = large_colored_txt();
    let mut renderer = AnsiRenderer::new();
    let mut app = create_app(create_pad_state(create_mem_terminal()));
    c.bench_function("AnsiRenderer::update_text", move |b| {
        b.iter(|| {
            renderer.update_text(&txt, &mut app.app_state.resources);
        })
    });

    // # draw dom
    let txt = large_colored_txt();
    let mut renderer = AnsiRenderer::new();
    let mut app = create_app(create_pad_state(create_mem_terminal()));
    renderer.update_text(&txt, &mut app.app_state.resources);
    c.bench_function("AnsiRenderer::dom", move |b| {
        b.iter(|| renderer.dom::<Mock>())
    });
}

criterion_group!(benches, ansi_rendering);
criterion_main!(benches);

fn create_app(pad: PadState<Mock, ()>) -> App<Mock> {
    App::new(Mock(AppValue::new(pad)), AppConfig::default()).unwrap()
}

fn create_pad_state(term: MemoryTerminal) -> PadState<Mock, ()> {
    let repl = repl_with_term!(term);

    let repl = repl
        .push_input_str("for _ in std::iter::repeat(0) { std::thread::sleep_ms(2); }")
        .unwrap_err(); // get ready for eval stage

    PadState::new(repl, Arc::new(RwLock::new(())))
}

fn create_mem_terminal() -> MemoryTerminal {
    let term = MemoryTerminal::with_size(Size {
        lines: 1000,
        columns: 1000,
    }); // large term to really push string creation (should be ~1MB)

    term.push_input(&format!("{}", &large_colored_txt())); // output some coloured stuff

    term
}

fn large_colored_txt() -> String {
    std::iter::repeat(cstr())
        .take(10) // TODO Increase this once I speed up update_text (try to get 995)
        .fold(String::new(), |mut acc, x| {
            acc.push_str(&x);
            acc.push('\n');
            acc
        })
}

fn cstr() -> String {
    format!(
        "{}{}{}{}{}{}{}{}{}{}{}",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.".red(),
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.".blue(),
        "Donec vel metus nec nisl ultrices cursus.".green(),
        "In in enim eget felis elementum consectetur et nec nisi.".purple(),
        "Morbi vel sapien consectetur, tristique sem id, facilisis purus.".yellow(),
        "Vivamus bibendum nisi ac lacus euismod hendrerit vel ac lacus.".red(),
        " Nulla scelerisque ipsum eu lacus dignissim, a tempus arcu egestas.".white(),
        "Nulla scelerisque ipsum eu lacus dignissim, a tempus arcu egestas.".bright_red(),
        "Praesent lobortis quam sed erat egestas, et tincidunt erat rutrum.".bright_white(),
        "Nullam maximus mauris a ultricies blandit.".bright_green(),
        "Morbi eget neque eget neque viverra mollis in id lacus.".bright_purple(),
    )
}

struct Mock(AppValue<PadState<Mock, ()>>);

impl Layout for Mock {
    fn layout(&self, _: LayoutInfo<Self>) -> Dom<Self> {
        Dom::div()
    }
}

use std::borrow::{Borrow, BorrowMut};
impl Borrow<AppValue<PadState<Mock, ()>>> for Mock {
    fn borrow(&self) -> &AppValue<PadState<Mock, ()>> {
        &self.0
    }
}

impl BorrowMut<AppValue<PadState<Mock, ()>>> for Mock {
    fn borrow_mut(&mut self) -> &mut AppValue<PadState<Mock, ()>> {
        &mut self.0
    }
}
