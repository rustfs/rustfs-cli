use std::collections::HashMap;
// use std::env;
use regex::Regex;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::Mutex;
// use serde_json::to_string;
// use url::Url;
use lazy_static::lazy_static;

use crate::cmd::aliaslist;
use crate::cmd::configx::load_config_v10;

// 全局变量存储配置目录和别名配置映射
static mut MC_CUSTOM_CONFIG_DIR: Option<String> = None;
lazy_static! {
    static ref ALIAS_TO_CONFIG_MAP: Mutex<HashMap<String, AliasConfigV10>> =
        Mutex::new(HashMap::new());
}

// 配置结构体
#[derive(Debug, Clone)]
struct AliasConfigV10 {
    url: String,
    access_key: Option<String>,
    secret_key: Option<String>,
    session_token: Option<String>,
    api: String,
    src: String,
}

// 设置自定义配置目录
pub fn set_mc_config_dir(config_dir: String) {
    unsafe {
        MC_CUSTOM_CONFIG_DIR = Some(config_dir);
    }
}

// 获取 MinIO 客户端配置目录
pub fn get_mc_config_dir() -> Result<String, io::Error> {
    unsafe {
        if let Some(ref dir) = MC_CUSTOM_CONFIG_DIR {
            return Ok(dir.clone());
        }
    }

    let home_dir = dirs::home_dir().ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        "Home directory not found",
    ))?;
    let config_dir = home_dir.join(default_mc_config_dir());
    Ok(config_dir.to_str().unwrap().to_string())
}

// 返回默认的 mc 配置目录
pub fn default_mc_config_dir() -> String {
    if cfg!(windows) {
        let cmd = std::env::args().next().unwrap_or_default();
        let cmd_base = Path::new(&cmd)
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap();
        return format!("{}\\", cmd_base);
    }
    //format!(".rustfs-cli/".to_string);
    ".rustfs-cli".to_string()
    //format!(".{}/", std::env::args().next().unwrap_or_default())
}

// 强制获取配置目录，如果出错则退出
pub fn must_get_mc_config_dir() -> String {
    get_mc_config_dir().unwrap_or_else(|e| {
        eprintln!("Unable to get mcConfigDir: {}", e);
        std::process::exit(1);
    })
}

// 创建 MinIO 客户端配置目录
fn create_mc_config_dir() -> Result<(), io::Error> {
    let config_dir = get_mc_config_dir()?;
    std::fs::create_dir_all(&config_dir)?;
    Ok(())
}

// 获取配置路径
pub fn get_mc_config_path() -> Result<String, io::Error> {
    unsafe {
        if let Some(ref dir) = MC_CUSTOM_CONFIG_DIR {
            return Ok(format!("{}/{}", dir, global_mc_config_file()));
        }
    }
    let config_dir = get_mc_config_dir()?;
    Ok(format!("{}/{}", config_dir, global_mc_config_file()))
}

// 强制获取配置路径，如果出错则退出
fn must_get_mc_config_path() -> String {
    get_mc_config_path().unwrap_or_else(|e| {
        eprintln!("Unable to get mcConfigPath: {}", e);
        std::process::exit(1);
    })
}

// 保存配置
pub fn save_mc_config(config: &AliasConfigV10) -> Result<(), io::Error> {
    create_mc_config_dir()?;
    let config_path = must_get_mc_config_path();
    fs::write(config_path, format!("{:?}", config))?; // 示例，将配置内容序列化保存
    Ok(())
}

// 检查配置是否存在
fn is_mc_config_exists() -> bool {
    if let Ok(config_path) = get_mc_config_path() {
        Path::new(&config_path).exists()
    } else {
        false
    }
}

// 清理别名
fn clean_alias(alias: &str) -> String {
    alias.trim_end_matches(&['/', '\\'][..]).to_string()
}

// 验证别名是否合法
fn is_valid_alias(alias: &str) -> bool {
    Regex::new(r"^[a-zA-Z][a-zA-Z0-9-_]*$")
        .unwrap()
        .is_match(alias)
}

// 从环境变量中获取别名配置
fn expand_alias_from_env(env_url: &str) -> Option<AliasConfigV10> {
    let (parsed_url, access_key, secret_key, session_token) = parse_env_url_str(env_url)?;
    Some(AliasConfigV10 {
        url: parsed_url,
        access_key: Some(access_key),
        secret_key: Some(secret_key),
        session_token: Some(session_token),
        api: "S3v4".to_string(),
        src: "env".to_string(),
    })
}

// 从环境变量解析 URL
fn parse_env_url_str(env_url: &str) -> Option<(String, String, String, String)> {
    let host_keys = Regex::new(r"^(https?://)(.*?):(.*)@(.*?)$").unwrap();
    let host_key_tokens = Regex::new(r"^(https?://)(.*?):(.*?):(.*)@(.*?)$").unwrap();

    if let Some(caps) = host_key_tokens.captures(env_url) {
        let access_key = caps[2].to_string();
        let secret_key = caps[3].to_string();
        let session_token = caps[4].to_string();
        let parsed_url = format!("{}{}", &caps[1], &caps[5]);
        Some((parsed_url, access_key, secret_key, session_token))
    } else if let Some(caps) = host_keys.captures(env_url) {
        let access_key = caps[2].to_string();
        let secret_key = caps[3].to_string();
        let parsed_url = format!("{}{}", &caps[1], &caps[4]);
        Some((parsed_url, access_key, secret_key, "".to_string()))
    } else {
        None
    }
}

// 从文件读取别名
fn read_aliases_from_file(env_config_file: &str) -> Result<(), io::Error> {
    let file = fs::File::open(env_config_file)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let env_line = line?;
        let strs: Vec<&str> = env_line.splitn(2, '=').collect();
        if strs.len() != 2 {
            eprintln!("Parsing error at {}", env_line);
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Parsing error"));
        }
        let alias = strs[0].trim_start_matches("MC_HOST_").to_string();
        if alias.is_empty() {
            eprintln!("Parsing error at {}", env_line);
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Parsing error"));
        }
        if let Some(alias_config) = expand_alias_from_env(strs[1]) {
            //unsafe {
            let mut alias_map = ALIAS_TO_CONFIG_MAP.lock().unwrap();
            alias_map.insert(alias, alias_config);
        }
    }
    Ok(())
}

fn global_mc_config_file() -> &'static str {
    "config.json"
}

pub fn alias_list(_name: &str) {
    //set_mc_config_dir("/home/ldy".to_string());
    // 示例：读取配置并打印
    if is_mc_config_exists() {
        //println!("Config exists at: {}", must_get_mc_config_path());

        match load_config_v10() {
            Ok(config) => {
                // 成功获取配置，处理 config
                //aliaslist::build_alias_message(_name, config);
                if _name.is_empty() {
                    aliaslist::list_aliases(&config.aliases, None);
                } else {
                    aliaslist::list_aliases(&config.aliases, Some(_name));
                }

                //println!("配置加载成功: {:?}", config);
            }
            Err(e) => {
                // 发生错误，处理错误
                println!("配置加载失败: {:?} {}", e, must_get_mc_config_path());
            }
        }

        // if let Err(e) = read_aliases_from_file("/home/ldy/config.json") {
        //     eprintln!("Failed to read aliases from file: {}", e);
        // } else {

        // }
    } else {
        println!("not exist: {}", must_get_mc_config_path());
        //println!("Config does not exist.");
    }
}

// pub fn main() {
//     set_mc_config_dir("/home/ldy".to_string());
//     // 示例：读取配置并打印
//     if is_mc_config_exists() {
//         println!("Config exists at: {}", must_get_mc_config_path());

//         match load_config_v10() {
//             Ok(config) => {
//                 // 成功获取配置，处理 config
//                 println!("配置加载成功: {:?}", config);
//             }
//             Err(e) => {
//                 // 发生错误，处理错误
//                 println!("配置加载失败: {:?}", e);
//             }
//         }

//         // if let Err(e) = read_aliases_from_file("/home/ldy/config.json") {
//         //     eprintln!("Failed to read aliases from file: {}", e);
//         // } else {

//         // }

//     } else {
// 	    println!("not exist: {}", global_mc_config_file());
//         //println!("Config does not exist.");
//     }
// }
