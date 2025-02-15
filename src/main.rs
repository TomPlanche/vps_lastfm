use async_lastfm::lastfm_handler::{LastFMHandler, TrackLimit};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let handler = LastFMHandler::new("tom_planche");

    let path = handler
        .export_recent_play_counts(TrackLimit::Limited(100))
        .await?;

    println!("Exported play counts to: {}", path);

    Ok(())
}
