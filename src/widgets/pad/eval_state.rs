use crate::prelude::*;
use linefeed::memory::MemoryTerminal;

type Read<D, R> = Repl<repl::Read, MemoryTerminal, D, R>;
type Eval<D, R> = repl::Evaluating<MemoryTerminal, D, R>;

enum EvalStateVariant<D, R> {
    Read(Read<D, R>),
    Eval(Eval<D, R>),
}

pub struct EvalState<D, R> {
    variant: Option<EvalStateVariant<D, R>>,
}

impl<D, R> EvalState<D, R> {
    pub fn new(repl: Read<D, R>) -> Self {
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

    pub fn take_read(&mut self) -> Option<Read<D, R>> {
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

    pub fn take_eval(&mut self) -> Option<Eval<D, R>> {
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

    pub fn put_read(&mut self, repl: Read<D, R>) {
        if self.variant.is_some() {
            panic!(
                "found some variant, inidicating a broken state. has a put call been called twice?"
            );
        }

        self.variant = Some(EvalStateVariant::Read(repl));
    }

    pub fn put_eval(&mut self, repl: Eval<D, R>) {
        if self.variant.is_some() {
            panic!(
                "found some variant, inidicating a broken state. has a put call been called twice?"
            );
        }

        self.variant = Some(EvalStateVariant::Eval(repl));
    }
}
