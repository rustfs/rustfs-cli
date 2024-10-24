mod cmd {
    pub mod admin;
    pub mod alias;
    pub mod aliaslist;
    pub mod aliasremove;
    pub mod aliasset;
    pub mod cmd;
    pub mod config;
    pub mod configx;
    pub mod main;
    pub mod tofu;
}
use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();
    cmd::main::main(args);
    //cmd::tofu::main();
    // cmd::aliaslist::main();

    // cmd::config::main();
}
