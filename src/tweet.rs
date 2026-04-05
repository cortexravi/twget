use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::json;

use crate::browser::{cookie_js, last_eval, run_batch};

#[derive(Serialize, serde::Deserialize)]
pub struct TweetOutput {
    pub id: String,
    pub author: String,
    pub text: String,
    pub created_at: String,
}

pub fn extract_id(url_or_id: &str) -> &str {
    if url_or_id.starts_with("http") {
        url_or_id
            .split('/')
            .next_back()
            .unwrap_or(url_or_id)
            .split('?')
            .next()
            .unwrap_or(url_or_id)
    } else {
        url_or_id
    }
}

const EXTRACT_JS: &str = r#"JSON.stringify(
    Array.from(document.querySelectorAll('article[data-testid="tweet"]')).map(a => {
        const link = a.querySelector('a[href*="/status/"]');
        const id = link?.href?.match(/\/status\/(\d+)/)?.[1];
        const authorEl = a.querySelector('[data-testid="User-Name"] a');
        const author = authorEl?.href?.match(/x\.com\/([^/?]+)/)?.[1];
        const text = a.querySelector('[data-testid="tweetText"]')?.innerText;
        const time = a.querySelector('time')?.getAttribute('datetime');
        return { id, author, text, created_at: time };
    }).filter(t => t.id && t.text)
)"#;

pub fn cmd_tweet(auth_token: &str, ct0: &str, url_or_id: &str, text_only: bool) -> Result<()> {
    let id = extract_id(url_or_id);
    let url = format!("https://x.com/i/status/{}", id);

    let commands = json!([
        ["open", "https://x.com"],
        ["eval", cookie_js(auth_token, ct0)],
        ["open", url],
        ["wait", "article[data-testid='tweet']"],
        ["eval", EXTRACT_JS]
    ]);

    let results = run_batch(&commands)?;
    let raw = last_eval(&results).context("No tweet data returned")?;
    let tweets: Vec<TweetOutput> = serde_json::from_str(raw).context("Failed to parse tweet")?;
    let tweet = tweets.into_iter().next().context("Tweet not found")?;

    if text_only {
        println!("{}", tweet.text);
    } else {
        println!("{}", serde_json::to_string_pretty(&tweet)?);
    }

    Ok(())
}

pub fn cmd_thread(auth_token: &str, ct0: &str, url_or_id: &str, text_only: bool) -> Result<()> {
    let id = extract_id(url_or_id);
    let url = format!("https://x.com/i/status/{}", id);

    let commands = json!([
        ["open", "https://x.com"],
        ["eval", cookie_js(auth_token, ct0)],
        ["open", url],
        ["wait", "article[data-testid='tweet']"],
        ["eval", EXTRACT_JS]
    ]);

    let results = run_batch(&commands)?;
    let raw = last_eval(&results).context("No tweet data returned")?;
    let tweets: Vec<TweetOutput> = serde_json::from_str(raw).context("Failed to parse thread")?;

    if text_only {
        for (i, t) in tweets.iter().enumerate() {
            println!("{}. @{}: {}", i + 1, t.author, t.text);
            println!();
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&tweets)?);
    }

    Ok(())
}
