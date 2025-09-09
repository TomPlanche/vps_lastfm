use async_lastfm::lastfm_handler::{LastFMHandler, Period, TrackLimit};
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
/// * `handler` - A reference to the `LastFMHandler` instance.
/// * `destination_folder` - The folder where the JSON file will be exported.
async fn fetch_recent_play_counts(handler: &LastFMHandler, destination_folder: &str) {
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

            if let Err(e) = handler
                .update_recent_play_counts(
                    TrackLimit::Limited(100),
                    &format!("{destination_folder}/recent_play_counts.json"),
                )
                .await
            {
                eprintln!("Failed to export recent play counts: {e:?}");
            }
        }
    }
}

/// Fetches the current track from Last.fm and exports it to a JSON file.
///
/// # Arguments
/// * `handler` - A reference to the `LastFMHandler` instance.
/// * `destination_folder` - The folder where the JSON file will be saved.
async fn fetch_current_track(handler: &LastFMHandler, destination_folder: &str) {
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

            if let Err(e) = handler
                .update_recent_play_counts(
                    TrackLimit::Limited(1),
                    &format!("{destination_folder}/currently_listening.json"),
                )
                .await
            {
                eprintln!("Failed to export current track: {e:?}");
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
async fn update_top_tracks_gist(handler: &LastFMHandler, username: &str, cfg: &Config) {
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

            let limit = TrackLimit::Limited(5);
            let period = Some(Period::Month);

            match handler.get_user_top_tracks(limit, period).await {
                Ok(mut top_tracks) => {
                    top_tracks.sort_by(|a, b| b.playcount.cmp(&a.playcount));

                    let content = format_top_tracks_markdown(username, &top_tracks);

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

    let handler = LastFMHandler::new(&cfg.last_fm_username);
    let destination_folder = cfg.destination_folder.clone();

    tokio::join!(
        fetch_recent_play_counts(&handler, &destination_folder),
        fetch_current_track(&handler, &destination_folder),
        update_top_tracks_gist(&handler, &cfg.last_fm_username, &cfg)
    );
}
