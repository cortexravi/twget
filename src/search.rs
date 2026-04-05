use anyhow::{bail, Context, Result};
use chrono::{Duration, Utc};
use serde_json::json;

use crate::browser::{cookie_js, last_eval, run_batch, url_encode};
use crate::tweet::TweetOutput;

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

pub fn cmd_search(
    auth_token: &str,
    ct0: &str,
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

    let search_url = format!("https://x.com/search?q={}&f=live", url_encode(&full_query));

    let extract_js = format!(
        r#"JSON.stringify(
    Array.from(document.querySelectorAll('article[data-testid="tweet"]'))
        .slice(0, {limit})
        .map(a => {{
            const link = a.querySelector('a[href*="/status/"]');
            const id = link?.href?.match(/\/status\/(\d+)/)?.[1];
            const authorEl = a.querySelector('[data-testid="User-Name"] a');
            const author = authorEl?.href?.match(/x\.com\/([^/?]+)/)?.[1];
            const text = a.querySelector('[data-testid="tweetText"]')?.innerText;
            const time = a.querySelector('time')?.getAttribute('datetime');
            return {{ id, author, text, created_at: time }};
        }})
        .filter(t => t.id && t.text)
)"#,
        limit = limit
    );

    let commands = json!([
        ["open", "https://x.com"],
        ["eval", cookie_js(auth_token, ct0)],
        ["open", search_url],
        ["wait", "article[data-testid='tweet']"],
        ["eval", extract_js]
    ]);

    let results = run_batch(&commands)?;
    let raw = last_eval(&results).context("No search results returned")?;
    let tweets: Vec<TweetOutput> =
        serde_json::from_str(raw).context("Failed to parse search results")?;

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
