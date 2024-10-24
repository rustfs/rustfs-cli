use super::{alias, configx};
use std::io::{self};
use std::error::Error;

use crate::cmd::config;
// use crate::cmd::alias;

use super::{tofu};




#[derive(Debug)]
struct AliasNotFoundError;

impl std::fmt::Display for AliasNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Alias not found")
    }
}

impl Error for AliasNotFoundError {}

pub fn remove_alias(alias: &str) ->  Result<(), Box<dyn Error>>{
	// 加载配置
	let mut mc_cfg_v10 = configx::load_config_v10().map_err(|err| {
	    eprintln!(
		"Unable to load config `{}`.",
		config::must_get_mc_config_dir()
	    );
	    err
	})?;

	if mc_cfg_v10.aliases.contains_key(alias) {
		// 存在，则移除
		mc_cfg_v10.aliases.remove(alias);
		
		// 保存配置
		configx::save_config_v10(&mc_cfg_v10).map_err(|err| {
		    eprintln!(
			"Unable to update hosts in config version `{}`.",
			config::must_get_mc_config_dir()
		    );
		    err
		})?;
		Ok(())
	    } else {
		// 不存在，返回错误
		Err(Box::new(AliasNotFoundError))
	    }
    }