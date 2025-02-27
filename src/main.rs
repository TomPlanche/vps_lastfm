use async_lastfm::lastfm_handler::{LastFMHandler, TrackLimit};
use chrono::Utc;
use cron::Schedule;
use std::str::FromStr;
use tokio::time::{sleep, Duration};

///
/// # `fetch_recent_play_counts`
/// Fetches the recent play counts from Last.fm and exports them to a JSON file.
///
/// ## Arguments
/// * `handler` - A reference to the LastFMHandler instance.
async fn fetch_recent_play_counts(handler: &LastFMHandler) {
    let expression = "0 0/1 * * * *"; // Every minutes
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
                .update_recent_play_counts(TrackLimit::Limited(100), "data/recent_play_counts.json")
                .await
            {
                eprintln!("Failed to export recent play counts: {e:?}");
            }
        }
    }
}

///
/// # `fetch_current_track`
/// Fetches the current track from Last.fm and exports it to a JSON file.
///
/// ## Arguments
/// * `handler` - A reference to the LastFMHandler instance.
async fn fetch_current_track(handler: &LastFMHandler) {
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
                .update_recent_play_counts(TrackLimit::Limited(1), "data/currently_listening.json")
                .await
            {
                eprintln!("Failed to export current track: {e:?}");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let handler = LastFMHandler::new("tom_planche");

    tokio::join!(
        fetch_recent_play_counts(&handler),
        fetch_current_track(&handler)
    );
}
