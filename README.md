# twget

A minimal Rust CLI for fetching tweets from Twitter/X — no official API key required. Uses cookie-based authentication via your existing Twitter account.

## Installation

```bash
cargo install --git https://github.com/cortexravi/twget
```

Or build from source:

```bash
git clone https://github.com/cortexravi/twget
cd twget
cargo build --release
# Binary at ./target/release/twget
```

## Authentication

### Option 1: Config file (recommended)

Create `~/.config/twget/config.toml`:

```toml
[twitter]
username = "your_username"
email = "your_email@example.com"
password = "your_password"
```

```bash
mkdir -p ~/.config/twget
chmod 700 ~/.config/twget
# create the file, then:
chmod 600 ~/.config/twget/config.toml
```

### Option 2: Environment variables

```bash
export TWITTER_USERNAME="your_username"
export TWITTER_EMAIL="your_email@example.com"
export TWITTER_PASSWORD="your_password"
```

Config file takes precedence. Individual fields can be mixed — e.g. `username` and `email` in the config file, `TWITTER_PASSWORD` as an env var.

### Session caching

After the first login, cookies are cached to `~/.twget/cookies.json` (mode 0600). Subsequent runs skip re-authentication until the session expires.

## Usage

### Fetch a single tweet

```bash
twget tweet https://twitter.com/user/status/1234567890
twget tweet 1234567890

# Plain text output
twget tweet 1234567890 --text
```

**JSON output:**
```json
{
  "id": "1234567890",
  "author": "username",
  "text": "Tweet content here",
  "created_at": "2026-04-04T21:00:00Z"
}
```

---

### Fetch a full thread

Fetches the original tweet plus its complete reply chain.

```bash
twget thread https://twitter.com/user/status/1234567890
twget thread 1234567890

# Readable plain text (numbered)
twget thread 1234567890 --text
```

**JSON output:** array of tweet objects in order.

---

### Search tweets

Search by keyword, hashtag, or phrase.

```bash
twget search "rolling puts"
twget search "#0DTE" --limit 50
twget search "theta gang" --since 24h
twget search "building in public" --since 7d --limit 10

# Plain text output
twget search "0DTE" --text
```

**Flags:**
- `--limit <n>` — max results to return (default: 20)
- `--since <duration>` — filter to tweets within this window (`24h`, `7d`, `30d`)
- `--text` — plain readable output instead of JSON

---

## Output format

All subcommands default to **JSON** output for easy piping into other tools. Use `--text` for human-readable output.

**Tweet object schema:**
```json
{
  "id": "string",
  "author": "string",
  "text": "string",
  "created_at": "ISO8601 string"
}
```

---

## Dependencies

- [`agent-twitter-client`](https://crates.io/crates/agent-twitter-client) — cookie-based Twitter internal API client
- [`clap`](https://crates.io/crates/clap) — CLI argument parsing
- [`tokio`](https://crates.io/crates/tokio) — async runtime
- [`serde_json`](https://crates.io/crates/serde_json) — JSON serialization
- [`toml`](https://crates.io/crates/toml) — config file parsing

---

## Notes

- This tool uses Twitter's **internal (unofficial) API**. It may break if Twitter changes its internals.
- Usage is subject to Twitter's Terms of Service. Intended for personal and research use.
- `--since` validation is performed after authentication — passing an invalid duration will error after login.
