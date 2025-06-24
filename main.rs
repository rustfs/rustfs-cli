mod cmd {
    pub mod admin;
    pub mod alias;
    pub mod aliasexport;
    pub mod aliaslist;
    pub mod aliasremove;
    pub mod aliasset;
    //pub mod clientadmin;
    pub mod cmd;
    pub mod config;
    pub mod configx;
    pub mod cp;
    pub mod find;
    pub mod ls;
    pub mod lsmain;
    pub mod main;
    pub mod mb;
    pub mod put;
    pub mod rb;
    pub mod rm;
    pub mod stat;
    pub mod tofu;
}

mod clientadmin;
mod infocommands;
use std::env;
mod s3;
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    cmd::main::main(args).await;
}
