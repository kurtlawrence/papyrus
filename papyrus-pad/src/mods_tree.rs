use super::*;
use azul::app::AppStateNoData;
use azul::callbacks::DefaultCallback;
use std::borrow::BorrowMut;

impl<T, D> PadState<T, D>
where
    T: 'static + BorrowMut<AppValue<PadState<T, D>>>,
    D: 'static + Send + Sync,
{
    fn on_mouse_down(
        &mut self,
        app: &mut AppStateNoData<T>,
        event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        let (idx, _) = event.get_index_in_parent(event.hit_dom_node)?;

        let repl = self.repl.brw_read()?;

        let path = repl.data.mods_map().keys().nth(idx)?;

        let line = format!(".mod switch {}", path.display());

        self.set_line_input(line)?;
        self.read_input(app)
    }

    cb!(priv_on_mouse_down, on_mouse_down);
}

pub struct ReplModulesTree;

impl ReplModulesTree {
    pub fn dom<T, D>(pad: &AppValue<PadState<T, D>>, info: &mut LayoutInfo<T>) -> Dom<T>
    where
        T: 'static + BorrowMut<AppValue<PadState<T, D>>>,
        D: 'static + Send + Sync,
    {
        let mut container = Dom::div().with_class("mods-tree");

        if let Some(repl) = pad.repl.brw_read() {
            let ptr = StackCheckedPointer::new(pad);

            let md_cbid = info
                .window
                .add_callback(ptr, DefaultCallback(PadState::<T, D>::priv_on_mouse_down));

            for path in repl.data.mods_map().keys() {
                let mut s = String::new();

                let comps = path.iter().count().saturating_sub(1);
                for _ in 0..comps {
                    s.push_str("|_");
                }

                path.iter()
                    .last()
                    .and_then(|x| x.to_str())
                    .map(|x| s.push_str(x));

                let mut item = Dom::label(s).with_class("mods-tree-item");

                item.add_default_callback_id(On::MouseDown, md_cbid);

                container.add_child(item);
            }
        }

        container
    }
}
