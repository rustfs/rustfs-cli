use clap::Subcommand;

#[derive(Subcommand)]
pub enum AdminCommands {
    List {
        #[arg(short, long)]
        verbose: bool,
    },
    Set,
    //     Get
}

// 处理 run 命令的逻辑
pub fn handle_admin_commands(subcommand: &AdminCommands) {
    match subcommand {
        AdminCommands::List { verbose } => {
            if *verbose {
                println!("Starting with verbose output...");
            } else {
                println!("Starting...");
            }
        }
        AdminCommands::Set => {
            println!("Stopping...");
        }
    }
}
