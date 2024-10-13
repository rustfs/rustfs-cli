use std::io::{self};

use crate::cmd::alias;
use crate::cmd::config;

use super::{configx, tofu};

// struct AliasConfigV10 {
//     url: String,
//     access_key: String,
//     secret_key: String,
//     api: String,
//     path: String,
// }

// #[derive(Debug, Clone)]
// struct AliasMessage {
//     alias: String,
//     url: String,
//     access_key: String,
//     secret_key: String,
//     api: String,
//     path: String,
// }

// #[derive(Debug)]
// struct McConfigV10 {
//     aliases: HashMap<String, configx::AliasConfigV10>,
// }

// impl McConfigV10 {
//     fn new() -> Self {
//         McConfigV10 {
//             aliases: HashMap::new(),
//         }
//     }
// }

// fn must_get_mc_config_path() -> String {
//     // 假设配置文件路径的函数
//     String::from("/path/to/mc/config")
// }

// fn load_mc_config() -> io::Result<McConfigV10> {
//     // 模拟加载配置文件
//     match fs::read_to_string(must_get_mc_config_path()) {
//         Ok(_config_content) => {
//             // 这里应该有配置的解析逻辑
//             Ok(McConfigV10::new())
//         }
//         Err(err) => Err(err),
//     }
// }

// fn save_mc_config(mc_cfg_v10: &McConfigV10) -> io::Result<()> {
//     // 模拟保存配置文件的逻辑
//     fs::write(must_get_mc_config_path(), format!("{:?}", mc_cfg_v10))
// }

pub async fn main_set_alias(
    alias: &str,
    url: &str,
    ak: &str,
    sk: &str,
) -> io::Result<alias::AliasMessage> {
    let alias_config = configx::AliasConfigV10 {
        url: String::from(url),
        access_key: String::from(ak),
        secret_key: String::from(sk),
        session_token: None,       // 可选字段，可以使用 None
        api: String::from("S3v4"), // 假设 API 版本是 v4
        path: String::from(""),
        license: Some(String::from("license-key")), // 可选字段，提供值时使用 Some
        api_key: None,                              // 可选字段，没有值时使用 None
        src: Some(String::from("source-info")),     // 可选字段，提供值时使用 Some
    };
    match tofu::check_bucket_permissions(url, ak, sk, "us-east-1").await {
        Ok(true) => set_alias(alias, alias_config),
        Ok(false) => {
            todo!();
        }
        Err(_) => {
            todo!()
        }
    }
}

fn set_alias(
    alias: &str,
    alias_cfg_v10: configx::AliasConfigV10,
) -> io::Result<alias::AliasMessage> {
    // 加载配置
    let mut mc_cfg_v10 = configx::load_config_v10().map_err(|err| {
        eprintln!(
            "Unable to load config `{}`.",
            config::must_get_mc_config_dir()
        );
        err
    })?;

    mc_cfg_v10
        .aliases
        .insert(alias.to_string(), alias_cfg_v10.clone());

    // 保存配置
    configx::save_config_v10(&mc_cfg_v10).map_err(|err| {
        eprintln!(
            "Unable to update hosts in config version `{}`.",
            config::must_get_mc_config_dir()
        );
        err
    })?;

    // 返回别名消息
    Ok(alias::AliasMessage {
        alias: alias.to_string(),
        url: alias_cfg_v10.url,
        access_key: alias_cfg_v10.access_key,
        secret_key: alias_cfg_v10.secret_key,
        api: alias_cfg_v10.api,
        path: alias_cfg_v10.path,
    })
}
