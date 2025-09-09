# VPS LastFM Data Fetcher

A lightweight Rust service that periodically fetches your Last.fm data and writes it to disk, while also updating a GitHub Gist with your top tracks on a schedule. It is designed to run unattended (e.g., on a VPS) and keep your data and gist up-to-date.

## What it does

The application runs three concurrent jobs:
- Fetches your last 100 played tracks every minute
- Tracks your currently playing song every 5 seconds
- Updates a GitHub Gist with your top tracks every hour (minute 0)

## Features

- 🎵 Real-time Last.fm data fetching
- 📊 Stores listening history in JSON format
- 📝 Hourly GitHub Gist update with your top tracks
- ⚡ Concurrent async jobs with robust scheduling
- 🔄 Automatic periodic updates

## Output files

The application will create two JSON files in your configured `DESTINATION_FOLDER`:
- `recent_play_counts.json`: your last 100 played tracks
- `currently_listening.json`: your currently playing track

## Update top tracks gist

An hourly job fetches your Last.fm top tracks and updates a configured GitHub Gist file with a Markdown section like:

```
<username>'s top listened tracks (refreshed hourly)

- [Track — Artist](url) — 123 plays — 1h 2m 3s
- [Track — Artist](url) — 87 plays — 45m 10s
```

Details:
- Runs hourly at minute 0 via cron-like scheduling
- Uses period: last month, and limit: top 5 tracks
- Sorts by play count (descending) and estimates total listened time per track
- Updates an existing gist file via the GitHub API

### Required environment variables

Set these in your `.env` (see `.env.example`):
- `LAST_FM_USERNAME`: your Last.fm username
- `DESTINATION_FOLDER`: where to write JSON files
- `GITHUB_TOKEN`: personal access token with `gist` scope
- `GIST_ID`: the target gist ID to update
- `GIST_FILENAME`: filename within the gist (defaults to `top-tracks.md` if unset)

## Setup

### Prerequisites

- Rust (latest stable)
- A Last.fm account
- A GitHub account and an existing Gist (for the gist feature)

### Install & run

1. Copy `.env.example` to `.env` and fill in values.
2. Build and run:

```
cargo run --release
```

The service will start the three scheduled jobs and continue running.

## Scheduling summary

- Recent plays: every minute (`0 0/1 * * * *`)
- Currently playing: every 5 seconds (`0/5 * * * * *`)
- Top tracks gist: hourly at minute 0 (`0 0 * * * *`)

## Notes

- The gist update logs errors (e.g., auth or network) but does not crash the service.
- `GIST_FILENAME` is optional; if not set, `top-tracks.md` is used.
