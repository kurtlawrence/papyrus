#[macro_use]
extern crate papyrus;

use papyrus::cmds::CommandResult;
use papyrus::cmdtree::{Builder, BuilderChain};
use papyrus::run::RunCallbacks;

fn main() {
    // Build a REPL that will use a String as the persistent app_data.
    let mut repl = repl!(String);

    // Inject our custom commands.
    repl.data.with_cmdtree_builder(custom_cmds()).unwrap();

    // Create the persistent data.
    let mut app_data = String::new();

    // Run the REPL and collect all the output.
    let output = repl.run(RunCallbacks::new(&mut app_data)).unwrap();

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
        .add_action("echo", "repeat back input after command", |writer, args| {
            writeln!(writer, "{}", args.join(" ")).ok();
            CommandResult::Empty
        })
        .begin_class("case", "change case of app_data")
        .add_action("upper", "make app_data uppercase", |_, _| {
            CommandResult::<String>::app_data_fn(|app_data, _repldata, _| {
                *app_data = app_data.to_uppercase();
                String::new()
            })
        })
        .add_action("lower", "make app_data lowercase", |_, _| {
            CommandResult::<String>::app_data_fn(|app_data, _repldata, _| {
                *app_data = app_data.to_lowercase();
                String::new()
            })
        })
        .end_class()
        .unwrap()
}
