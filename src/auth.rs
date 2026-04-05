use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Default)]
struct Config {
    twitter: Option<TwitterConfig>,
}

#[derive(Deserialize)]
struct TwitterConfig {
    auth_token: Option<String>,
    ct0: Option<String>,
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("twget")
        .join("config.toml")
}

fn load_config() -> Config {
    let path = config_path();
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Config::default()
    }
}

fn resolve(config_val: Option<String>, env_key: &str) -> Option<String> {
    config_val.or_else(|| std::env::var(env_key).ok())
}

pub fn get_credentials() -> Result<(String, String)> {
    let config = load_config();
    let tw = config.twitter.as_ref();
    let auth_token = resolve(tw.and_then(|t| t.auth_token.clone()), "TWITTER_AUTH_TOKEN")
        .context("auth_token not set — add it to ~/.config/twget/config.toml or set TWITTER_AUTH_TOKEN")?;
    let ct0 = resolve(tw.and_then(|t| t.ct0.clone()), "TWITTER_CT0")
        .context("ct0 not set — add it to ~/.config/twget/config.toml or set TWITTER_CT0")?;
    Ok((auth_token, ct0))
}
