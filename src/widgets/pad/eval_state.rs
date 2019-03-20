use crate::prelude::*;
use linefeed::memory::MemoryTerminal;

type Read<D> = Repl<repl::Read, MemoryTerminal, D>;
type Eval<D> = repl::Evaluating<MemoryTerminal, D>;

enum EvalStateVariant<D> {
    Read(Read<D>),
    Eval(Eval<D>),
}

pub struct EvalState<D> {
    variant: Option<EvalStateVariant<D>>,
}

impl<D> EvalState<D> {
    pub fn new(repl: Read<D>) -> Self {
        EvalState {
            variant: Some(EvalStateVariant::Read(repl)),
        }
    }

    pub fn is_read(&self) -> bool {
        if self.variant.is_none() {
            panic!("found none variant, inidicating a broken state. has a take call been called twice?");
        }

        match self.variant.as_ref().expect("should be some") {
            EvalStateVariant::Read(_) => true,
            EvalStateVariant::Eval(_) => false,
        }
    }

    pub fn is_eval(&self) -> bool {
        if self.variant.is_none() {
            panic!("found none variant, inidicating a broken state. has a take call been called twice?");
        }

        match self.variant.as_ref().expect("should be some") {
            EvalStateVariant::Read(_) => false,
            EvalStateVariant::Eval(_) => true,
        }
    }

    pub fn take_read(&mut self) -> Option<Read<D>> {
        if self.variant.is_none() {
            panic!("found none variant, inidicating a broken state. has a take call been called twice?");
        }

        match self.variant.take().expect("should be some") {
            EvalStateVariant::Read(repl) => Some(repl),
            EvalStateVariant::Eval(repl) => {
                self.variant = Some(EvalStateVariant::Eval(repl));
                None
            }
        }
    }

    pub fn take_eval(&mut self) -> Option<Eval<D>> {
        if self.variant.is_none() {
            panic!("found none variant, inidicating a broken state. has a take call been called twice?");
        }

        match self.variant.take().expect("should be some") {
            EvalStateVariant::Read(repl) => {
                self.variant = Some(EvalStateVariant::Read(repl));
                None
            }
            EvalStateVariant::Eval(repl) => Some(repl),
        }
    }

    pub fn put_read(&mut self, repl: Read<D>) {
        if self.variant.is_some() {
            panic!(
                "found some variant, inidicating a broken state. has a put call been called twice?"
            );
        }

        self.variant = Some(EvalStateVariant::Read(repl));
    }

    pub fn put_eval(&mut self, repl: Eval<D>) {
        if self.variant.is_some() {
            panic!(
                "found some variant, inidicating a broken state. has a put call been called twice?"
            );
        }

        self.variant = Some(EvalStateVariant::Eval(repl));
    }
}
