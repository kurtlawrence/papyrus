# Benching Coverage

---

## Evaluation

---

Evaluation is consdered the time it takes for a `Repl` to run the `.evaluate()` function. This includes (at the moment):

- Parsing the code (as a program),
- Building the additional statements,
- Building a `SourceFile`,
- Compiling the source file,
- Runnning the executable,
- Adding the additional statements to the repl.

This is effectively the complete cycle of an evaluation. There is an additional step of cleaning the repl but that is hard to test on its own without also benching the code to make it dirty in the first place.

| Bench Name   | Description                                                         | Clean Repl Each Loop | Testing                            |
| ------------ | ------------------------------------------------------------------- | :------------------: | ---------------------------------- |
| eval_01      | Evaluate a single statement                                         | Yes                  | Speed of evaluation cycle          |
| eval_10_sing | Evaluate 10 statements, in single function call                     | Yes                  | Speed of evaluation cycle          |
| eval_20_sing | Evaluate 20 statements, in single function call                     | Yes                  | Speed of evaluation cycle          |
| eval_prog    | Progressively evaluate statements (do initial eval outside of loop) | No                   | Speed progressive evaluation calls |
