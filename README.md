# twget

A minimal Rust CLI for fetching tweets from Twitter/X — no official API key required.

## Installation

```bash
cargo install --git https://github.com/cortexravi/twget
```

Or build from source:

```bash
git clone https://github.com/cortexravi/twget
cd twget
cargo build --release
```

## Authentication

### Option 1: Browser cookies (recommended)

Twitter blocks programmatic logins. The reliable approach is to extract two cookies from your logged-in browser session and put them in the config file.

**How to get your cookies:**
1. Open Twitter/X in your browser and log in
2. Open DevTools → Application → Cookies → `https://x.com`
3. Copy the values of `auth_token` and `ct0`

**`~/.config/twget/config.toml`:**
```toml
[twitter]
auth_token = "your_auth_token_value"
ct0 = "your_ct0_value"
```

```bash
mkdir -p ~/.config/twget && chmod 700 ~/.config/twget
# create the file, then:
chmod 600 ~/.config/twget/config.toml
```

These two cookies are all that's needed. No session caching required with this method.

---

### Option 2: Username/password (fallback, unreliable)

Twitter frequently returns 401 on programmatic logins. Use this only if cookie auth isn't an option.

```toml
[twitter]
username = "your_username"
email = "your_email@example.com"
password = "your_password"
```

Or via environment variables: `TWITTER_USERNAME`, `TWITTER_EMAIL`, `TWITTER_PASSWORD`.

On success, the session is cached to `~/.twget/cookies.json` (mode 0600).

---

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

```bash
twget thread https://twitter.com/user/status/1234567890
twget thread 1234567890

# Readable plain text (numbered)
twget thread 1234567890 --text
```

JSON output: array of tweet objects in order.

---

### Search tweets

```bash
twget search "rolling puts"
twget search "#0DTE" --limit 50
twget search "theta gang" --since 24h
twget search "building in public" --since 7d --limit 10

# Plain text output
twget search "0DTE" --text
```

**Flags:**
- `--limit <n>` — max results (default: 20)
- `--since <duration>` — filter window: `24h`, `7d`, `30d`
- `--text` — plain output instead of JSON

---

## Output format

All subcommands default to JSON. Use `--text` for human-readable output.

```json
{
  "id": "string",
  "author": "string",
  "text": "string",
  "created_at": "ISO8601 string"
}
```

---

## Notes

- Uses Twitter's **internal (unofficial) API** via [`agent-twitter-client`](https://crates.io/crates/agent-twitter-client). May break if Twitter changes internals.
- Intended for personal and research use.
