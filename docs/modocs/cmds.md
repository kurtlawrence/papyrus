Extendable commands for REPL.

The REPL makes use of the crate [`cmdtree`](https://crates.io/crates/cmdtree) to handle commands
that can provide additional functionality over just a Rust REPL.
A command is prefixed by a colon (`:`) and a number of defaults. To see the commands that are
included, type `:help`.

# Common Commands

There are three common commands, `help`, `cancel` or `c`, and `exit`, which can be invoked in any
class.

| cmd      | action                                          |
| -------- | ----------------------------------------------- |
| `help`   | displays help information for the current class |
| `cancel` | moves the class back to the command root        |
| `exit`   | quit the REPL                                   |

Other commands are context based off the command tree, they can be invoked with something similar
to `a nested command action` syntax.

# Extending Commands
## Setup

This tutorial works through the example at
[`papyrus/examples/custom-cmds.rs`](https://github.com/kurtlawrence/papyrus/blob/master/papyrus/examples/custom-cmds.rs).

To begin, start a binary project with the following scaffolding in the main source code. We define
a `custom_cmds` function that will be used to build our custom commands. To highlight the
versatility of commands, the REPL is configured to have a persistent app data through a `String`.
Notice also the method to alter the prompt name through the `Builder::new` method.

```rust,no_run
#[macro_use]
extern crate papyrus;

use papyrus::cmdtree::{Builder, BuilderChain};
use papyrus::cmds::CommandResult;

# #[cfg(not(feature = "runnable"))]
# fn main() {}

# #[cfg(feature = "runnable")]
fn main() {
    // Build a REPL that will use a String as the persistent app_data.
    let mut repl = repl!(String);

    // Inject our custom commands.
    repl.data.with_cmdtree_builder(custom_cmds()).unwrap();

    // Create the persistent data.
    let mut app_data = String::new();

    // Run the REPL and collect all the output.
    let output = repl.run(papyrus::run::RunCallbacks::new(&mut app_data)).unwrap();

    // Print the output.
    println!("{}", output);
}

// Define our custom commands.
// The CommandResult takes the same type as the app_data,
// in this instance it is a String. We could define it as
// a generic type but then it loses resolution to interact with
// the app_data through commands.
fn custom_cmds() -> Builder<CommandResult<String>> {
    // The string defines the name and the prompt that will be used.
    Builder::new("custom-cmds-app")
}
```

## Echo

Let's begin with a simple echo command. This command takes the data after the command and prints it
to screen. All these commands will be additions to the `Builder::new`.
Adding the following action with `add_action` method, the arguments are written to the `Write`able
`writer`. The REPL provides the writer and so captures the output. `args` is passed through as a
slice of string slices, `cmdtree` provides this, and are always split on word boundaries.
Finally, `CommandResult::Empty` is returned which `papyrus` further processes. `Empty` won't do
anything but the API provides alternatives.

```rust
# extern crate papyrus;
# use papyrus::cmdtree::BuilderChain;
# use papyrus::cmds::CommandResult;
# type Builder = papyrus::cmdtree::Builder<CommandResult<String>>;
Builder::new("custom-cmds-app")
    .add_action("echo", "repeat back input after command", |writer, args| {
    writeln!(writer, "{}", args.join(" ")).ok();
    CommandResult::Empty
    })
    .unwrap()
# ;
```

Now when the binary is run the REPL runs as usual. If `:help` is entered you should see the
following output.

```text
[lib] custom-cmds-app=> :help
help -- prints the help messages
cancel | c -- returns to the root class
exit -- sends the exit signal to end the interactive loop
Classes:
    edit -- Edit previous input
    mod -- Handle modules
Actions:
    echo -- repeat back input after command
    mut -- Begin a mutable block of code
[lib] custom-cmds-app=>
```

The `echo` command exists as a root level action, with the help message displayed. Try calling
`:echo Hello, world!` and see what it does!


## Alter app data

To extend what the commands can do, lets create a command set that can convert the persistent app
data case.
The actual actions are nested under a 'class' named `case`. This means to invoke the action, one
would call it through `:case upper` or `:case lower`.

```rust
# extern crate papyrus;
# use papyrus::cmdtree::BuilderChain;
# use papyrus::cmds::CommandResult;
# type Builder = papyrus::cmdtree::Builder<CommandResult<String>>;
Builder::new("custom-cmds-app")
    .add_action("echo", "repeat back input after command", |writer, args| {
    writeln!(writer, "{}", args.join(" ")).ok();
    CommandResult::Empty
    })
    .begin_class("case", "change case of app_data")
    .add_action("upper", "make app_data uppercase", |_, _|
    CommandResult::<String>::app_data_fn(|app_data, _repldata, _| {
        *app_data = app_data.to_uppercase();
        String::new()
        })
    )
        .add_action("lower", "make app_data lowercase", |_, _|
    CommandResult::<String>::app_data_fn(|app_data, _repldata, _| {
        *app_data = app_data.to_lowercase();
        String::new()
        })
    )
    .end_class()
    .unwrap()
# ;
```

An example output is below. To inject some data into the persistent app data, a mutable code block
must be entered first.

```text
[lib] papyrus=> :mut
beginning mut block
[lib] custom-cmds-app-mut=> app_data.push_str("Hello, world!")
finished mutating block: ()
[lib] custom-cmds-app=> app_data.as_str()
custom-cmds-app [out0]: "Hello, world!"
[lib] custom-cmds-app=> :case upper
[lib] custom-cmds-app=> app_data.as_str()
custom-cmds-app [out1]: "HELLO, WORLD!"
[lib] custom-cmds-app=> :case lower
[lib] custom-cmds-app=> app_data.as_str()
custom-cmds-app [out2]: "hello, world!"
```

