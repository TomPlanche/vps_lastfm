# VPS LastFM Data Fetcher

A simple Rust application that periodically fetches and stores Last.fm listening data. The application runs two concurrent tasks:
- Fetches your last 100 played tracks every minute
- Tracks your currently playing song every 5 seconds

## Features

- 🎵 Real-time Last.fm data fetching
- 📊 Stores listening history in JSON format
- ⚡ Concurrent data collection
- 🔄 Automatic periodic updates

## Setup

### Prerequisites

- Rust (latest stable version)
- A Last.fm account

The application will create two JSON files in your specified destination folder:
- `recent_play_counts.json`: Contains your last 100 played tracks
- `currently_listening.json`: Contains your currently playing track

## Configuration

- Update the username in `.env` file.
- Update the destination folder in `.env` file.
