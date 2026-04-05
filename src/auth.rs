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
    /// Preferred auth: browser cookie value for auth_token
    auth_token: Option<String>,
    /// Required alongside auth_token: CSRF token from browser cookies
    ct0: Option<String>,
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
    let config = load_config();
    let tw = config.twitter.as_ref();

    let auth_token = resolve(tw.and_then(|t| t.auth_token.clone()), "TWITTER_AUTH_TOKEN");
    let ct0 = resolve(tw.and_then(|t| t.ct0.clone()), "TWITTER_CT0");

    if let (Some(auth_token), Some(ct0)) = (auth_token, ct0) {
        // Preferred path: use browser cookies directly, no login() required
        let cookie_str = format!(
            "auth_token={}; ct0={}",
            auth_token, ct0
        );
        scraper
            .set_from_cookie_string(&cookie_str)
            .await
            .context("Failed to set cookies from auth_token/ct0")?;
    } else {
        // Fallback path: cached session or username/password login
        let path = cookie_path();
        if path.exists() {
            let cookie_str = fs::read_to_string(&path).context("Failed to read cookie cache")?;
            if scraper.set_from_cookie_string(&cookie_str).await.is_err() {
                login_and_save(&mut scraper, tw).await?;
            }
        } else {
            login_and_save(&mut scraper, tw).await?;
        }
    }

    Ok(scraper)
}

async fn login_and_save(scraper: &mut Scraper, tw: Option<&TwitterConfig>) -> Result<()> {
    let username = resolve(tw.and_then(|t| t.username.clone()), "TWITTER_USERNAME")
        .context("No auth_token/ct0 found. Falling back to login, but username is not set.\nAdd auth_token and ct0 to ~/.config/twget/config.toml (recommended), or set TWITTER_USERNAME.")?;

    let password = resolve(tw.and_then(|t| t.password.clone()), "TWITTER_PASSWORD")
        .context("TWITTER_PASSWORD not set")?;

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
