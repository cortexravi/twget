use agent_twitter_client::scraper::Scraper;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

// --- Config file ---

#[derive(Deserialize, Default)]
struct Config {
    twitter: Option<TwitterConfig>,
}

#[derive(Deserialize)]
struct TwitterConfig {
    username: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("twget").join("config.toml")
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

/// Resolve a credential: config file first, env var fallback.
fn resolve(config_val: Option<String>, env_key: &str) -> Option<String> {
    config_val.or_else(|| std::env::var(env_key).ok())
}

// --- Cookie cache ---

fn cookie_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".twget").join("cookies.json")
}

// --- Public API ---

pub async fn get_scraper() -> Result<Scraper> {
    let mut scraper = Scraper::new().await.context("Failed to initialize scraper")?;

    let path = cookie_path();
    if path.exists() {
        let cookie_str = fs::read_to_string(&path).context("Failed to read cookie cache")?;
        if scraper.set_from_cookie_string(&cookie_str).await.is_err() {
            login_and_save(&mut scraper).await?;
        }
    } else {
        login_and_save(&mut scraper).await?;
    }

    Ok(scraper)
}

async fn login_and_save(scraper: &mut Scraper) -> Result<()> {
    let config = load_config();
    let tw = config.twitter.as_ref();

    let username = resolve(
        tw.and_then(|t| t.username.clone()),
        "TWITTER_USERNAME",
    )
    .context(
        "Twitter username not set — add it to ~/.config/twget/config.toml or set TWITTER_USERNAME",
    )?;

    let password = resolve(
        tw.and_then(|t| t.password.clone()),
        "TWITTER_PASSWORD",
    )
    .context(
        "Twitter password not set — add it to ~/.config/twget/config.toml or set TWITTER_PASSWORD",
    )?;

    let email = resolve(tw.and_then(|t| t.email.clone()), "TWITTER_EMAIL");

    scraper
        .login(username, password, email, None)
        .await
        .context("Twitter login failed")?;

    let path = cookie_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create ~/.twget directory")?;
    }

    let cookie_str = scraper
        .get_cookie_string()
        .await
        .context("Failed to retrieve session cookies")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)
            .context("Failed to open cookie file")?;
        f.write_all(cookie_str.as_bytes())
            .context("Failed to write cookies")?;
    }
    #[cfg(not(unix))]
    {
        fs::write(&path, &cookie_str).context("Failed to save cookie cache")?;
    }

    Ok(())
}
