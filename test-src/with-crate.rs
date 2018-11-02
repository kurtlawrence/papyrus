extern crate argparse;

fn one() {
	println!("Hello, world!",);
	println!("{}", std::env::current_dir().unwrap().to_string_lossy());
}
