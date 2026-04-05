use agent_twitter_client::scraper::Scraper;
use anyhow::{Context, Result};
use serde::Serialize;

#[derive(Serialize)]
pub struct TweetOutput {
    pub id: String,
    pub author: String,
    pub text: String,
    pub created_at: String,
}

/// Extract tweet ID from a full URL or return input unchanged if already an ID.
pub fn extract_id(url_or_id: &str) -> &str {
    if url_or_id.starts_with("http") {
        url_or_id
            .split('/')
            .last()
            .unwrap_or(url_or_id)
            .split('?')
            .next()
            .unwrap_or(url_or_id)
    } else {
        url_or_id
    }
}

pub fn tweet_to_output(tweet: &agent_twitter_client::models::Tweet) -> TweetOutput {
    TweetOutput {
        id: tweet.id.clone().unwrap_or_default(),
        author: tweet.username.clone().unwrap_or_default(),
        text: tweet.text.clone().unwrap_or_default(),
        created_at: tweet.created_at.clone().unwrap_or_default(),
    }
}

pub async fn cmd_tweet(scraper: &Scraper, url_or_id: &str, text_only: bool) -> Result<()> {
    let id = extract_id(url_or_id);
    let tweet = scraper
        .get_tweet(id)
        .await
        .context("Failed to fetch tweet")?;

    let out = tweet_to_output(&tweet);

    if text_only {
        println!("{}", out.text);
    } else {
        println!("{}", serde_json::to_string_pretty(&out)?);
    }

    Ok(())
}

pub async fn cmd_thread(scraper: &mut Scraper, url_or_id: &str, text_only: bool) -> Result<()> {
    let id = extract_id(url_or_id);
    let root = scraper
        .get_tweet(id)
        .await
        .context("Failed to fetch root tweet")?;

    let mut tweets = vec![tweet_to_output(&root)];

    // Append any thread replies attached to the root.
    // Note: root.thread is a shallow list of direct replies from the API response —
    // it does not recursively walk nested reply chains.
    for t in &root.thread {
        tweets.push(tweet_to_output(t));
    }

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
