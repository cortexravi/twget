mod auth;
mod browser;
mod search;
mod tweet;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "twget", about = "Fetch tweets without an API key")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch a single tweet by URL or ID
    Tweet {
        /// Tweet URL (https://x.com/user/status/ID) or bare ID
        url_or_id: String,
        /// Print only the tweet text
        #[arg(long)]
        text: bool,
    },
    /// Fetch a tweet and its reply chain
    Thread {
        /// Tweet URL or bare ID
        url_or_id: String,
        /// Print readable plain text (numbered tweets)
        #[arg(long)]
        text: bool,
    },
    /// Search tweets by keyword or hashtag
    Search {
        /// Search query
        query: String,
        /// Maximum number of results (default: 20)
        #[arg(long, default_value = "20")]
        limit: i32,
        /// Only show tweets since this duration ago (e.g. 24h, 7d)
        #[arg(long)]
        since: Option<String>,
        /// Print readable plain text output
        #[arg(long)]
        text: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let (auth_token, ct0) = auth::get_credentials()?;

    match cli.command {
        Commands::Tweet { url_or_id, text } => {
            tweet::cmd_tweet(&auth_token, &ct0, &url_or_id, text)?;
        }
        Commands::Thread { url_or_id, text } => {
            tweet::cmd_thread(&auth_token, &ct0, &url_or_id, text)?;
        }
        Commands::Search {
            query,
            limit,
            since,
            text,
        } => {
            search::cmd_search(&auth_token, &ct0, &query, limit, since.as_deref(), text)?;
        }
    }

    Ok(())
}
