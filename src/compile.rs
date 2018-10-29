use super::*;
use failure::{Context, ResultExt};
use std::io::Write;
use std::path::Path;
use std::process;

/// Runs `cargo build`, then runs the executable  from the given directory. Stdin and Stdout are inherited (allowing live updating of progress).
/// Waits for process to finish and returns the `Output` of the process.
pub fn compile_and_run<P: AsRef<Path>, Q: AsRef<Path>>(
	src: &SourceFile,
	compile_dir: &P,
	working_dir: &Q,
) -> Result<process::Output, Context<String>> {
	let compile_dir = compile_dir.as_ref();
	build_compile_dir(src, &compile_dir)?;

	let working_dir = working_dir.as_ref();
	let status = process::Command::new("cargo")
		.current_dir(compile_dir)
		.arg("build")
		.output()
		.context("cargo command failed to start, is rust installed?".to_string())?;
	if !status.status.success() {
		return Err(Context::new(format!(
			"Build failed\n{}",
			String::from_utf8_lossy(&status.stderr)
		)));
	}

	let mut exe = format!(
		"{}/target/debug/{}",
		compile_dir.to_string_lossy(),
		src.file_name
	);
	if cfg!(windows) {
		exe.push_str(".exe");
	}

	process::Command::new(&exe)
		.current_dir(working_dir)
		.output()
		.context(format!(
			"failed to run {} in dir {}",
			exe,
			working_dir.to_string_lossy()
		))
}

/// Constructs the compile directory with the given main source file contents.
/// Expects `SourceFileType::Rs` to define a `main()` function.
/// `SourceFileType::Rscript` will encase code in a `main()` function.
fn build_compile_dir<P: AsRef<Path>>(
	source: &SourceFile,
	compile_dir: &P,
) -> Result<(), Context<String>> {
	let compile_dir = compile_dir.as_ref();
	let mut main_file = create_file_and_dir(&compile_dir.join("src/main.rs"))?;
	let mut cargo_file = create_file_and_dir(&compile_dir.join("Cargo.toml"))?;
	let cargo = cargotoml_contents(source);
	let content = main_contents(source);
	main_file
		.write_all(content.as_bytes())
		.context("failed writing contents of main.rs".to_string())?;
	cargo_file
		.write_all(cargo.as_bytes())
		.context("failed writing contents of Cargo.toml".to_string())?;
	Ok(())
}

fn cargotoml_contents(source: &SourceFile) -> String {
	format!(
		r#"[package]
name = "{pkg_name}"
version = "0.1.0"

[dependencies]
{crates}
"#,
		pkg_name = source.file_name,
		crates = source
			.crates
			.iter()
			.map(|c| format!(r#"{} = "*""#, c.cargo_name))
			.collect::<Vec<_>>()
			.join("\n")
	)
}

fn main_contents(source: &SourceFile) -> String {
	format!(
		r#"
{crates}

{src}
"#,
		crates = source
			.crates
			.iter()
			.map(|c| c.src_line.clone())
			.collect::<Vec<_>>()
			.join("\n"),
		src = match source.file_type {
			SourceFileType::Rs => source.src.clone(),
			SourceFileType::Rscript => format!(
				r#"fn main() {{
	{}
}}"#,
				source.src
			),
		}
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_build_compile_dir() {
		let source = SourceFile {
			src: TEST_CONTENTS.to_string(),
			file_type: SourceFileType::Rs,
			file_name: "test-name".to_string(),
			crates: Vec::new(),
		};

		build_compile_dir(&source, &"tests/compile-dir/test-dir").unwrap();
		assert!(Path::new("tests/compile-dir/test-dir/src/main.rs").exists());
		assert!(Path::new("tests/compile-dir/test-dir/Cargo.toml").exists());

		fs::remove_dir_all("tests/compile-dir/test-dir").unwrap();
	}

	#[test]
	fn test_run() {
		use std::env;
		let dir = "tests/compile-dir/test-run";
		let source = SourceFile {
			src: TEST_CONTENTS.to_string(),
			file_type: SourceFileType::Rs,
			file_name: "test-name".to_string(),
			crates: Vec::new(),
		};
		compile_and_run(&source, &dir, &env::current_dir().unwrap()).unwrap();

		fs::remove_dir_all(dir).unwrap();
	}

	const TEST_CONTENTS: &str = "fn main() { println!(\"Hello, world!\"); }";
}
