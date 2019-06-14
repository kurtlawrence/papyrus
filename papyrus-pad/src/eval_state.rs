use papyrus::prelude::*;

type Read<D> = Repl<repl::Read, D>;
type Eval<D> = repl::Evaluating<D>;

enum EvalStateVariant<D> {
    Read(Read<D>),
    Eval(Eval<D>),
    None,
}

pub struct EvalState<D> {
    variant: EvalStateVariant<D>,
}

impl<D> EvalState<D> {
    pub fn new(repl: Read<D>) -> Self {
        EvalState {
            variant: EvalStateVariant::Read(repl),
        }
    }

    pub fn brw_read(&self) -> Option<&Read<D>> {
        match &self.variant {
            EvalStateVariant::Read(repl) => Some(repl),
            EvalStateVariant::Eval(_) => None,
            EvalStateVariant::None => None,
        }
    }

    pub fn take_read(&mut self) -> Option<Read<D>> {
        let v = std::mem::replace(&mut self.variant, EvalStateVariant::None);

        match v {
            EvalStateVariant::Read(r) => Some(r),
            _ => {
                self.variant = v;
                None
            }
        }
    }

    pub fn take_eval(&mut self) -> Option<Eval<D>> {
        let v = std::mem::replace(&mut self.variant, EvalStateVariant::None);

        match v {
            EvalStateVariant::Eval(e) => Some(e),
            _ => {
                self.variant = v;
                None
            }
        }
    }

    pub fn put_read(&mut self, repl: Read<D>) {
        self.variant = EvalStateVariant::Read(repl);
    }

    pub fn put_eval(&mut self, repl: Eval<D>) {
        self.variant = EvalStateVariant::Eval(repl);
    }
}
