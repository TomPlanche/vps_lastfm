use lastfm_client::{LastFmClient, api::Period};
use chrono::Utc;
use cron::Schedule;
use std::str::FromStr;
use tokio::time::{sleep, Duration};

mod update_gist;
use update_gist::{format_top_tracks_markdown, update_gist};

mod config;
use config::Config;

/// Fetches the recent play counts from Last.fm and exports them to a JSON file.
///
/// # Arguments
/// * `client` - A reference to the `LastFmClient` instance.
/// * `username` - The Last.fm username to fetch tracks for.
/// * `destination_folder` - The folder where the JSON file will be exported.
async fn fetch_recent_play_counts(client: &LastFmClient, username: &str, destination_folder: &str) {
    let expression = "0 0/1 * * * *"; // Every minute
    let schedule = Schedule::from_str(expression).expect("Failed to parse CRON expression");

    loop {
        let now = Utc::now();
        if let Some(next) = schedule.upcoming(Utc).take(1).next() {
            let until_next = next - now;
            sleep(Duration::from_secs(
                u64::try_from(until_next.num_seconds()).unwrap_or_default(),
            ))
            .await;

            match client.recent_tracks(username).limit(100).fetch().await {
                Ok(tracks) => {
                    let path = format!("{destination_folder}/recent_play_counts.json");
                    match serde_json::to_string_pretty(&tracks) {
                        Ok(json) => {
                            if let Err(e) = std::fs::write(&path, json) {
                                eprintln!("Failed to write recent play counts: {e:?}");
                            }
                        }
                        Err(e) => eprintln!("Failed to serialize recent play counts: {e:?}"),
                    }
                }
                Err(e) => eprintln!("Failed to fetch recent play counts: {e:?}"),
            }
        }
    }
}

/// Fetches the current track from Last.fm and exports it to a JSON file.
///
/// # Arguments
/// * `client` - A reference to the `LastFmClient` instance.
/// * `username` - The Last.fm username to fetch the current track for.
/// * `destination_folder` - The folder where the JSON file will be saved.
async fn fetch_current_track(client: &LastFmClient, username: &str, destination_folder: &str) {
    let expression = "0/5 * * * * *"; // Each 5 seconds
    let schedule = Schedule::from_str(expression).expect("Failed to parse CRON expression");

    loop {
        let now = Utc::now();
        if let Some(next) = schedule.upcoming(Utc).take(1).next() {
            let until_next = next - now;
            sleep(Duration::from_secs(
                u64::try_from(until_next.num_seconds()).unwrap_or_default(),
            ))
            .await;

            match client.recent_tracks(username).limit(1).fetch().await {
                Ok(tracks) => {
                    let path = format!("{destination_folder}/currently_listening.json");
                    match serde_json::to_string_pretty(&tracks) {
                        Ok(json) => {
                            if let Err(e) = std::fs::write(&path, json) {
                                eprintln!("Failed to write current track: {e:?}");
                            }
                        }
                        Err(e) => eprintln!("Failed to serialize current track: {e:?}"),
                    }
                }
                Err(e) => eprintln!("Failed to fetch current track: {e:?}"),
            }
        }
    }
}

/// Hourly job: fetch user top tracks and update the configured GitHub gist.
///
/// - Runs at `minute 0` of every hour using a cron schedule
/// - Fetches top tracks for the configured period and limit
/// - Renders Markdown via `format_top_tracks_markdown`
/// - Updates the gist content, logging (but not crashing) on errors
async fn update_top_tracks_gist(client: &LastFmClient, username: &str, cfg: &Config) {
    // Every hour, at minute 0
    let expression = "0 0 * * * *";
    let schedule = Schedule::from_str(expression).expect("Failed to parse CRON expression");

    loop {
        let now = Utc::now();
        if let Some(next) = schedule.upcoming(Utc).take(1).next() {
            let until_next = next - now;
            sleep(Duration::from_secs(
                u64::try_from(until_next.num_seconds()).unwrap_or_default(),
            ))
            .await;

            match client
                .top_tracks(username)
                .limit(5)
                .period(Period::Week)
                .fetch()
                .await
            {
                Ok(mut top_tracks) => {
                    top_tracks.sort_by(|a, b| b.playcount.cmp(&a.playcount));

                    let content = format_top_tracks_markdown(&top_tracks);

                    if let Err(e) = update_gist(
                        &content,
                        &cfg.github_token,
                        &cfg.gist_id,
                        &cfg.gist_filename,
                    )
                    .await
                    {
                        eprintln!("Failed to update gist: {e:?}");
                    }
                }
                Err(e) => eprintln!("Failed to fetch top tracks: {e:?}"),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let cfg = Config::load().expect("Missing or invalid configuration");
    cfg.ensure_destination_folder()
        .expect("Failed to ensure destination folder");

    let client = LastFmClient::new().expect("Failed to create Last.fm client");
    let username = cfg.last_fm_username.clone();
    let destination_folder = cfg.destination_folder.clone();

    tokio::join!(
        fetch_recent_play_counts(&client, &username, &destination_folder),
        fetch_current_track(&client, &username, &destination_folder),
        update_top_tracks_gist(&client, &username, &cfg)
    );
}
