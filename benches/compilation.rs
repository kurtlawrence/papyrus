#[macro_use]
extern crate criterion;

extern crate papyrus;

use criterion::Criterion;
use papyrus::*;

fn single_evaluations(c: &mut Criterion) {
	// eval_01
	c.bench_function("eval_01", |b| {
		b.iter(|| {
			let mut repl = Repl::new();
			repl.print = false;
			repl.clean();
			repl.evaluate("2+2").unwrap();
		})
	});
	// eval_10_sing
	c.bench_function("eval_10_sing", |b| {
		b.iter(|| {
			let mut repl = Repl::new();
			repl.print = false;
			repl.clean();
			repl.evaluate(STMTS_10).unwrap();
		})
	});
	// eval_20_sing
	c.bench_function("eval_20_sing", |b| {
		b.iter(|| {
			let mut repl = Repl::new();
			repl.print = false;
			repl.clean();
			repl.evaluate(STMTS_20).unwrap();
		})
	});
}

fn progressive_evaluation(c: &mut Criterion) {
	// eval_prog
	c.bench_function("eval_prog", |b| {
		let mut repl = Repl::new();
		repl.print = false;
		repl.clean();
		repl.evaluate("2+2").unwrap();
		b.iter(|| {
			repl.evaluate("2+2").unwrap();
		})
	});
}

criterion_group!{
name = singles;
config = Criterion::default().sample_size(10);
 targets = single_evaluations
 }

criterion_group!{
name = progressives;
config = Criterion::default().sample_size(15);
 targets = progressive_evaluation
 }

criterion_main!(singles, progressives);

const STMTS_10: &str = r#"let a = 1;
let b = 2;
let c = a * b;
let c = a * c + 10;
let a = a * b * c;
let mut s = String::from("Hello");
let a = a + b + c;
let c = a - b;
s.push_str(", world!");
s"#;

const STMTS_20: &str = r#"let a = 1;
let b = 2;
let c = a * b;
let c = a * c + 10;
let a = a * b * c;
let mut s = String::from("Hello ");
let a = a + b + c;
let c = a - b;
let d = a + b + c;
let e = a + b + c  + d;
let f = d - e;
let a = a - d - e;
let b = d - f;
s.push_str(&a.to_string());
s.push_str(&b.to_string());
s.push_str(&c.to_string());
s.push_str(&d.to_string());
s.push_str(&e.to_string());
s.push_str(&f.to_string());
s"#;
