use async_lastfm::lastfm_handler::{LastFMHandler, TrackLimit};
use chrono::Utc;
use cron::Schedule;
use std::str::FromStr;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let handler = LastFMHandler::new("tom_planche");

    let expression = "0 0/5 * * * *"; // Every 5 minutes
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
                .export_recent_play_counts(TrackLimit::Limited(100))
                .await
            {
                eprintln!("Failed to export recent play counts: {e:?}");
            }
        }
    }
}
