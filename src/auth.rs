use agent_twitter_client::scraper::Scraper;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

fn cookie_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".twget").join("cookies.json")
}

pub async fn get_scraper() -> Result<Scraper> {
    let mut scraper = Scraper::new().await.context("Failed to initialize scraper")?;

    let path = cookie_path();
    if path.exists() {
        let cookie_str = fs::read_to_string(&path).context("Failed to read cookie cache")?;
        if scraper.set_from_cookie_string(&cookie_str).await.is_err() {
            // Cached cookies are invalid — re-login
            login_and_save(&mut scraper).await?;
        }
    } else {
        login_and_save(&mut scraper).await?;
    }

    Ok(scraper)
}

async fn login_and_save(scraper: &mut Scraper) -> Result<()> {
    let username = std::env::var("TWITTER_USERNAME").context("TWITTER_USERNAME not set")?;
    let password = std::env::var("TWITTER_PASSWORD").context("TWITTER_PASSWORD not set")?;
    let email = std::env::var("TWITTER_EMAIL").ok();

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
    fs::write(&path, &cookie_str).context("Failed to save cookie cache")?;

    Ok(())
}
