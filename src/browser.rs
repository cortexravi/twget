use anyhow::{Context, Result};
use serde_json::Value;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn cookie_js(auth_token: &str, ct0: &str) -> String {
    format!(
        "document.cookie='auth_token={auth_token};domain=.x.com;path=/'; \
         document.cookie='ct0={ct0};domain=.x.com;path=/'"
    )
}

/// Run a batch of agent-browser commands, return the array of result objects.
pub fn run_batch(commands: &Value) -> Result<Vec<Value>> {
    let input = serde_json::to_string(commands)?;

    let mut child = Command::new("agent-browser")
        .args(["batch", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start agent-browser — is it installed?")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        anyhow::bail!(
            "agent-browser failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout)?;
    let results: Vec<Value> =
        serde_json::from_str(&stdout).context("Failed to parse agent-browser output")?;

    for r in &results {
        if r.get("success").and_then(|s| s.as_bool()) == Some(false) {
            if let Some(err) = r.get("error").and_then(|e| e.as_str()) {
                if !err.is_empty() {
                    anyhow::bail!("agent-browser command failed: {}", err);
                }
            }
        }
    }

    Ok(results)
}

/// Extract the JavaScript return value from the last eval command in batch results.
pub fn last_eval(results: &[Value]) -> Option<&str> {
    results
        .iter()
        .rev()
        .find(|r| {
            r.get("command")
                .and_then(|c| c.as_array())
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                == Some("eval")
        })
        .and_then(|r| r.pointer("/result/result"))
        .and_then(|v| v.as_str())
}

/// Simple percent-encoding for URL query parameters.
pub fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push('+'),
            _ => {
                use std::fmt::Write;
                write!(out, "%{:02X}", byte).unwrap();
            }
        }
    }
    out
}
