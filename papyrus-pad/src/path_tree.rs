use super::*;
use azul::prelude::*;

pub struct PathTree;

impl PathTree {
    pub fn dom<T, D>(pad: &AppValue<PadState<T, D>>) -> Dom<T> {
        let mut container = Dom::div().with_class("path-tree");

        if let Some(repl) = pad.repl.brw_repl() {
            for path in repl.data.file_map().keys() {
                let mut s = String::new();

                let comps = path.iter().count().saturating_sub(1);
                for _ in 0..comps {
                    s.push_str("|_");
                }

                path.iter()
                    .last()
                    .and_then(|x| x.to_str())
                    .map(|x| s.push_str(x));

                container.add_child(Dom::label(s).with_class("path-tree-item"));
            }
        }

        container
    }
}
