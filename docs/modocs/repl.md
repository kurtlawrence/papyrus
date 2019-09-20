The REPL takes the commands given and evaluates them,
setting a local variable such that the data can be continually referenced.

REPLs are state machines, consisting of a read, evaluate, and print states.
Each state will lead directly to the next with relevant methods.
Generally a user will only use the `.read` and `.eval` methods.
Calling `.run` will consume the REPL and block the thread until it exits.

There is functionality to pass data between the REPL and the application. This functionality is
detailed under the [linking module](linking).

You can replicate the `run` behaviour with a basic implementation:
```rust,no_run
use papyrus::prelude::*;

let mut repl = Repl::default();

loop {
    let readres = repl.read();
    match readres {
	ReadResult::Read(r) => repl = r,
	ReadResult::Eval(e) => {
	    let evalres = e.eval(&mut ());
	    match evalres.signal {
		Signal::Exit => break,
		Signal::None => (),
	    }
	    repl = evalres.repl.print().0;
	}
    }
}
```

