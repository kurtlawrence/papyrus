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

| Bench Name   | Description                                                         | Tests                              |
| ------------ | ------------------------------------------------------------------- | ---------------------------------- |
| eval_01      | Evaluate a single statement                                         | Speed of evaluation cycle          |
| eval_10_sing | Evaluate 10 statements, in single function call                     | Speed of evaluation cycle          |
| eval_20_sing | Evaluate 20 statements, in single function call                     | Speed of evaluation cycle          |
| eval_prog    | Progressively evaluate statements (do initial eval outside of loop) | Speed progressive evaluation calls |

## Parsing

---

Heavy parsing is almost entirely done in the `parse_program` function. Here we test using `libtest`.

## Compilation and Execution

---

Currently compilation is done through `cargo` and interacts with the filesystem.

| Bench Name                | Description                                 | Tests                                                                                                  |
| ------------------------- | ------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| cargo_fs_compile_clean_10 | Compiles 10 statements in a clean directory | Heaviest compilation is in a clean dir                                                                 |
| cargo_fs_compile_clean_20 | Compiles 20 statements in a clean directory | Heaviest compilation is in a clean dir                                                                 |
| cargo_fs_compile          | Compiles in a already compiled directory    | Compilation time should be low, tests the filesystem interactions with building compilation directory. |
| exe_fs_run                | Runs a simple program                       | Sanity check on runtime                                                                                |