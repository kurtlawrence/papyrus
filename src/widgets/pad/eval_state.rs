enum EvalStateVariant {
	Read(Repl<repl::Read, MemoryTerminal, (), linking::NoRef>),
	Eval(repl::Evaluating<MemoryTerminal, (), linking::NoRef>),
}

pub struct EvalState {
	variant: Option<EvalStateVariant>,
}

impl EvalState {
	pub fn new(repl: Repl<repl::Read, MemoryTerminal, (), linking::NoRef>) -> Self {
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

	pub fn take_read(&mut self) -> Option<Repl<repl::Read, MemoryTerminal, (), linking::NoRef>> {
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

	pub fn take_eval(&mut self) -> Option<repl::Evaluating<MemoryTerminal, (), linking::NoRef>> {
		if self.variant.is_none() {
			panic!("found none variant, inidicating a broken state. has a take call been called twice?");
		}

		match self.variant.take().expect("should be some") {
			EvalStateVariant::Read(repl) => {
				self.variant = Some(EvalStateVariant::Read(repl));
				None
			} ,
			EvalStateVariant::Eval(repl) => Some(repl)
		}
	}

	pub fn put_read(&mut self, repl: Repl<repl::Read, MemoryTerminal, (), linking::NoRef>) {
		if self.variant.is_some() {
			panic!("found some variant, inidicating a broken state. has a put call been called twice?");
		}

		self.variant = Some(EvalStateVariant::Read(repl));
	}

	pub fn put_eval(&mut self, repl: repl::Evaluating<MemoryTerminal, (), linking::NoRef>) {
		if self.variant.is_some() {
			panic!("found some variant, inidicating a broken state. has a put call been called twice?");
		}

		self.variant = Some(EvalStateVariant::Eval(repl));
	}
}