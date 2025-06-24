// use clap::{Arg, Command};
use std::collections::HashMap;
use std::fmt;

use super::config;
use super::configx::AliasConfigV10;

// 定义 alias 配置结构
//#[derive(Debug, Clone)]
// struct AliasConfig {
//     url: String,
//     access_key: Option<String>,
//     secret_key: Option<String>,
//     api: Option<String>,
//     path: Option<String>,
// }

// 定义 alias 信息结构
#[derive(Debug, Clone)]
struct AliasMessage {
    alias: String,
    url: String,
    access_key: String,
    secret_key: String,
    api: String,
    path: String,
}

impl fmt::Display for AliasMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\n\tURL: {}\n\tAccessKey: {}\n\tSecretKey: {}\n\tAPI: {}\n\tPath: {}\n",
            self.alias, self.url, self.access_key, self.secret_key, self.api, self.path
        )
    }
}

pub fn alias_list(name: &str) {
    config::alias_list(name);
}

// 列出所有 aliases
pub fn list_aliases(aliases: &HashMap<String, AliasConfigV10>, alias_name: Option<&str>) {
    if let Some(alias) = alias_name {
        if let Some(config) = aliases.get(alias) {
            let msg = build_alias_message(alias, config);
            println!("{}", msg);
        } else {
            println!("No such alias `{}` found.", alias);
        }
    } else {
        let mut alias_list: Vec<_> = aliases.keys().collect();
        alias_list.sort();
        for alias in alias_list {
            let config = &aliases[alias];
            let msg = build_alias_message(alias, config);
            println!("{}", msg);
        }
    }
}

// 构建 alias 消息
fn build_alias_message(alias: &str, config: &AliasConfigV10) -> AliasMessage {
    let access_key = &config.access_key;
    AliasMessage {
        alias: alias.to_string(),
        url: config.url.clone(),
        access_key: access_key.clone(),
        secret_key: config.secret_key.clone(),
        api: config.api.clone(),
        path: config.path.clone(),
    }
}

// pub fn main() {
//     // 定义命令行工具
//     let matches = Command::new("minio-cli")
//         .version("1.0")
//         .author("Your Name <you@example.com>")
//         .about("Manager client for MinIO and RustFS")
//         .subcommand(
//             Command::new("alias")
//                 .about("Manage server credentials")
//                 .subcommand(
//                     Command::new("list")
//                         .about("List aliases in configuration file")
//                         .arg(
//                             Arg::new("ALIAS")
//                                 .help("The alias to list")
//                                 .required(false)
//                         )
//                 )
//                 .subcommand(
//                     Command::new("remove")
//                         .about("Remove an alias from the configuration file")
//                         .arg(
//                             Arg::new("ALIAS")
//                                 .help("The alias to remove")
//                                 .required(true)
//                         )
//                 )
//         )
//         .get_matches();

//     // 假设的别名配置
//     let mut aliases: HashMap<String, AliasConfigV10> = HashMap::new();
//     aliases.insert(
//         "s3".to_string(),
//         AliasConfigV10 {
//             url: "https://s3.amazonaws.com".to_string(),
//             access_key: Some("ACCESS_KEY".to_string()),
//             secret_key: Some("SECRET_KEY".to_string()),
//             api: Some("S3v4".to_string()),
//             path: Some("/path/to/config".to_string()),
//         },
//     );

//     // 处理子命令
//     if let Some(("alias", sub_matches)) = matches.subcommand() {
//         match sub_matches.subcommand() {
//             Some(("list", list_matches)) => {
//                 let alias_name = list_matches.get_one::<String>("ALIAS").map(|s| s.as_str());
//                 list_aliases(&aliases, alias_name);
//             }
//             _ => {
//                 println!("Unknown alias subcommand.");
//             }
//         }
//     }
// }
