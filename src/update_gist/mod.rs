//! Utilities to format top tracks into Markdown and update a GitHub Gist.
//!
//! This module provides two primary capabilities:
//! - Formatting Last.fm top tracks into a concise Markdown list suitable for a profile or gist.
//! - Updating an existing GitHub Gist file with the rendered Markdown content.
//!
//! Environment variables expected by `update_gist`:
//! - `GITHUB_TOKEN`: Personal access token with `gist` scope.
//! - `GIST_ID`: Identifier of the target gist.
//! - `GIST_FILENAME`: File name within the gist to update (defaults to `top-tracks.md`).

use async_lastfm::types::TopTrack;
use std::fmt::Write as _;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};

/// Updates a GitHub Gist file with the provided content.
///
/// # Arguments
/// * `content` - The full text to write into the configured gist file.
///
/// # Errors
/// Returns an error if the environment variables are missing or if the API request fails.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` on success.
pub async fn update_gist(
    content: &str,
    github_token: &str,
    gist_id: &str,
    gist_filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("vps-lastfm-bot"));
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {github_token}"))?,
    );

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "files": {
            gist_filename: { "content": content }
        }
    });

    let url = format!("https://api.github.com/gists/{gist_id}");
    let resp = client
        .patch(url)
        .headers(headers)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Failed to update gist: {status} - {text}").into());
    }

    Ok(())
}

/// Formats seconds into a human-friendly string like `1h 23m 45s`.
///
/// # Arguments
/// * `total_seconds` - Total number of seconds to format.
///
/// # Returns
/// * `String` - Human-readable duration string.
fn format_seconds_hms(total_seconds: u64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    let mut parts: Vec<String> = Vec::new();
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 || hours > 0 {
        parts.push(format!("{minutes}m"));
    }
    parts.push(format!("{seconds}s",));

    parts.join(" ")
}

/// Formats tracks into Markdown bullet lines: `[Title — Artist](url) — N plays — T`.
///
/// # Arguments
/// * `items` - A list of `(markdown_title, url, playcount, total_listened_seconds)` tuples.
///
/// # Returns
/// * `String` - A multi-line string suitable for a gist/profile README.
pub fn format_tracks_markdown(items: &[(String, String, u64, u64)]) -> String {
    if items.is_empty() {
        return "No tracks found.".to_string();
    }

    let mut lines: Vec<String> = Vec::new();

    for (title_md, url, playcount, total_secs) in items {
        if *total_secs == 0 {
            let line = format!("- [{title_md}]({url}) — {playcount} plays");
            lines.push(line);
        } else {
            let duration = format_seconds_hms(*total_secs);
            let line = format!("- [{title_md}]({url}) — {playcount} plays — {duration}");
            lines.push(line);
        }
    }

    lines.join("\n")
}

/// Formats a list of Last.fm `TopTrack` into a Markdown section with a header and bullets.
///
/// - Sorts by `playcount` (descending)
/// - Computes an approximate total listened time: `duration * playcount`
/// - Prepends a title: `"<username>'s top listened tracks (refreshed hourly)"`
///
/// Returns a multi-line Markdown string.
pub fn format_top_tracks_markdown(username: &str, tracks: &[TopTrack]) -> String {
    let mut sorted: Vec<&TopTrack> = tracks.iter().collect();
    sorted.sort_by(|a, b| b.playcount.cmp(&a.playcount));

    let display_items: Vec<(String, String, u64, u64)> = sorted
        .iter()
        .map(|t| {
            let title_md = format!("{} — {}", t.name, t.artist.name);
            let url = t.url.clone();
            let playcount = u64::from(t.playcount);
            let duration_secs = u64::from(t.duration);
            let total_secs = duration_secs.saturating_mul(playcount);
            (title_md, url, playcount, total_secs)
        })
        .collect();

    let mut out = String::new();
    let _ = write!(
        out,
        "{username}'s top listened tracks (refreshed hourly)\n\n"
    );
    out.push_str(&format_tracks_markdown(&display_items));
    out
}
