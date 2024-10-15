mod  cmd {
	pub mod aliaslist;
	pub mod aliasset;
	pub mod config;
	pub mod main;
        pub mod configx;
	pub mod cmd;
	pub mod alias;
	pub mod admin;
	pub mod tofu;
}
use std::env;
fn main() {
	let args:Vec<String> = env::args().collect();
	cmd::main::main(args);
	//cmd::tofu::main();
	// cmd::aliaslist::main();

	// cmd::config::main();
}