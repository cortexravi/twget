use agent_twitter_client::search::SearchMode;
use agent_twitter_client::scraper::Scraper;
use anyhow::{bail, Context, Result};
use chrono::{Duration, Utc};

use crate::tweet::{tweet_to_output, TweetOutput};

fn parse_since(duration: &str) -> Result<String> {
    let dt = if let Some(h) = duration.strip_suffix('h') {
        let hours: i64 = h.parse().context("Invalid hours value")?;
        Utc::now() - Duration::hours(hours)
    } else if let Some(d) = duration.strip_suffix('d') {
        let days: i64 = d.parse().context("Invalid days value")?;
        Utc::now() - Duration::days(days)
    } else {
        bail!("Invalid duration '{}'. Use format like 24h or 7d", duration);
    };
    Ok(dt.format("%Y-%m-%d").to_string())
}

pub async fn cmd_search(
    scraper: &mut Scraper,
    query: &str,
    limit: i32,
    since: Option<&str>,
    text_only: bool,
) -> Result<()> {
    let mut full_query = query.to_string();
    if let Some(dur) = since {
        let since_date = parse_since(dur)?;
        full_query = format!("{} since:{}", full_query, since_date);
    }

    let response = scraper
        .search_tweets(&full_query, limit, SearchMode::Latest, None)
        .await
        .context("Search failed")?;

    let tweets: Vec<TweetOutput> = response.tweets.iter().map(tweet_to_output).collect();

    if text_only {
        for (i, t) in tweets.iter().enumerate() {
            println!("{}. @{} [{}]: {}", i + 1, t.author, t.created_at, t.text);
            println!();
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&tweets)?);
    }

    Ok(())
}
