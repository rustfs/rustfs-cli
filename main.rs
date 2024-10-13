mod  cmd {
	pub mod  cmd;
	pub mod main;
}
use std::env;
fn main() {
	let args:Vec<String> = env::args().collect();
	cmd::main::Main(args);
}