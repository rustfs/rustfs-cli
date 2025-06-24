use clap::Subcommand;

use super::{aliasexport, aliaslist, aliasremove, aliasset};

#[derive(Debug, Clone)]
pub struct AliasMessage {
    pub alias: String,
    pub url: String,
    pub access_key: String,
    pub secret_key: String,
    pub api: String,
    pub path: String,
}
#[derive(Subcommand)]
pub enum AliasCommands {
    #[command(about = "set a new alias to configuration file")]
    #[clap(visible_alias = "ls")]
    List {
        #[arg(help = "section")]
        alias_name: Option<String>,
    },
    #[clap(visible_alias = "s")]
    #[command(about = "list aliases in configuration file")]
    Set {
        // 定义 `alias` 名称
        alias: String,
        // 定义 `URL` 参数
        url: String,
        // 定义 `Access Key`
        access_key: String,
        // 定义 `Secret Key`
        secret_key: String,
    },
    #[clap(visible_alias = "rm")]
    #[command(about = "remove an alias from configuration file")]
    Remove { alias: String },
    #[clap(visible_alias = "i")]
    #[command(
        about = "import configuration info to configuration file from a JSON formatted string"
    )]
    Import,
    #[clap(visible_alias = "e")]
    #[command(about = "export configuration info to stdout")]
    Export { alias: String }, //     Get
}

// 处理 run 命令的逻辑
pub async fn handle_alias_commands(subcommand: &AliasCommands) {
    match subcommand {
        AliasCommands::List { alias_name } => {
            if let Some(ref name) = alias_name {
                println!("Listing alias: {}", name);
                aliaslist::alias_list(name);
            } else {
                aliaslist::alias_list("");
            }
        }
        AliasCommands::Set {
            alias,
            url,
            access_key,
            secret_key,
        } => {
            let _ = aliasset::main_set_alias(&alias, &url, &access_key, &secret_key).await;
        }
        AliasCommands::Export { alias } => {
            match aliasexport::export_alias(alias) {
                Ok(json_string) => {
                    // Successfully serialized the alias content
                    println!("{}", json_string);
                }
                Err(e) => {
                    // Handle the error, like alias not found or config loading error
                    eprintln!("Error: {}", e);
                }
            }
        }
        &AliasCommands::Import => {
            println!("Stopping...");
        }
        AliasCommands::Remove { alias } => {
            let ret = aliasremove::remove_alias(&alias);
            match ret {
                Ok(_) => {
                    println!("Alias '{}' removed successfully!", alias);
                }
                Err(e) => {
                    println!("Failed to remove alias '{}'. Error: {:?}", alias, e);
                }
            }
        }
    }
}
