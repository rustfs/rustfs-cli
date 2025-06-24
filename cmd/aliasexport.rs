use super::configx;
use crate::cmd::config;
use std::error::Error;

#[derive(Debug)]
struct AliasNotFoundError;

impl std::fmt::Display for AliasNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Alias not found")
    }
}

impl Error for AliasNotFoundError {}

pub fn export_alias(alias: &str) -> Result<String, Box<dyn Error>> {
    // 加载配置
    let mc_cfg_v10 = configx::load_config_v10().map_err(|err| {
        eprintln!(
            "Unable to load config `{}`.",
            config::must_get_mc_config_dir()
        );
        err
    })?;

    if let Some(content) = mc_cfg_v10.aliases.get(alias) {
        // 存在，返回内容
        let str = serde_json::to_string(content).unwrap();
        Ok(str) // assuming content is a String or can be cloned
    } else {
        // 不存在，返回 AliasNotFoundError
        Err(Box::new(AliasNotFoundError))
    }
}
