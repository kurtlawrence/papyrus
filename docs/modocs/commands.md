# Commands

>Back to [SUMMARY](./SUMMARY.md)

Current list of commands.

## help

Show help for commands.

`.help [text]`

> Args is another command, and will explicitly show the help for that command.

## exit

Exit repl.

`.exit`

> Exits the repl loop. Arguments are ignored.

## cancel -- c

Cancel the current input. This is useful for times when more input is expected but you would like to cancel it. (There is an edge case where a leading bracket [`)`] cannot be closed off, so being able to cancel is useful.

```sh
.cancel
.c
```

> Arguments are ignored.

## load

Loads a `*.rs` or `*.rscript` file as inputs.

`.load <filename>`

> See [File Interaction](./file-interaction.md).

Suggest or add a command on [github](https://github.com/kurtlawrence/papyrus).