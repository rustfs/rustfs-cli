use super::config;
use std::collections::HashMap;
// use std::process;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Error as IoError;
use std::sync::RwLock;
// use std::io;

// 默认 AccessKey 和 SecretKey
const DEFAULT_ACCESS_KEY: &str = "YOUR-ACCESS-KEY-HERE";
const DEFAULT_SECRET_KEY: &str = "YOUR-SECRET-KEY-HERE";

// 全局变量和缓存
static CACHE_CFG_V10: Lazy<RwLock<Option<ConfigV10>>> = Lazy::new(|| RwLock::new(None));

// 配置的结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasConfigV10 {
    pub url: String,
    #[serde(rename = "accessKey")]
    pub access_key: String,
    #[serde(rename = "secretKey")]
    pub secret_key: String,
    pub session_token: Option<String>,
    pub api: String,
    pub path: String,
    pub license: Option<String>,
    pub api_key: Option<String>,
    pub src: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigV10 {
    version: String,
    pub aliases: HashMap<String, AliasConfigV10>,
}

// 新建 ConfigV10
fn new_config_v10() -> ConfigV10 {
    ConfigV10 {
        version: "1.0".to_string(),
        aliases: HashMap::new(),
    }
}

impl ConfigV10 {
    // 设置别名
    fn set_alias(&mut self, alias: String, cfg: AliasConfigV10) {
        self.aliases.entry(alias).or_insert(cfg);
    }

    // 加载默认配置
    fn load_defaults(&mut self) {
        self.set_alias(
            "local".to_string(),
            AliasConfigV10 {
                url: "http://localhost:9000".to_string(),
                access_key: "".to_string(),
                secret_key: "".to_string(),
                api: "S3v4".to_string(),
                path: "auto".to_string(),
                session_token: None,
                license: None,
                api_key: None,
                src: None,
            },
        );

        self.set_alias(
            "s3".to_string(),
            AliasConfigV10 {
                url: "https://s3.amazonaws.com".to_string(),
                access_key: DEFAULT_ACCESS_KEY.to_string(),
                secret_key: DEFAULT_SECRET_KEY.to_string(),
                api: "S3v4".to_string(),
                path: "dns".to_string(),
                session_token: None,
                license: None,
                api_key: None,
                src: None,
            },
        );
    }
}

pub fn load_config_v10() -> Result<ConfigV10, IoError> {
    let cache = CACHE_CFG_V10.read().unwrap();

    // 如果有缓存，返回缓存值
    if let Some(ref config) = *cache {
        return Ok(config.clone());
    }

    // 解锁后加载配置文件
    drop(cache); // 显式释放读锁

    let config_path = config::get_mc_config_path()?;

    // 读取并解析 JSON 文件
    println!("config path is:{}", config_path);
    let config_data = fs::read_to_string(&config_path)?;
    let mut config_v10: ConfigV10 = serde_json::from_str(&config_data)?;

    // 加载默认值
    config_v10.load_defaults();

    // 缓存配置
    let mut cache = CACHE_CFG_V10.write().unwrap();
    *cache = Some(config_v10.clone());

    Ok(config_v10)
}

// 保存配置
pub fn save_config_v10(config_v10: &ConfigV10) -> Result<(), IoError> {
    match config::get_mc_config_path() {
        Ok(path) => {
            let config_data = serde_json::to_string_pretty(config_v10)?;
            fs::write(&path, config_data)?;

            // 更新缓存
            let mut cache = CACHE_CFG_V10.write().unwrap();
            *cache = Some(config_v10.clone());
        }
        Err(e) => {
            eprintln!("Error getting config path: {}", e);
        }
    }
    Ok(())
}
