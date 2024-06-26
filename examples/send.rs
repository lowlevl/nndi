use nndi::{ffmpeg, Source};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg_next::init()?;

    // Set-up the log and traces handler
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let source = Source::new(nndi::source::Config {
        name: "super source".into(),
        ..Default::default()
    })
    .await?;

    let timebase = ffmpeg::sys::AVRational { num: 1, den: 1 };
    let mut frame = ffmpeg::frame::Video::new(ffmpeg::format::Pixel::RGBA, 600, 360);
    frame.data_mut(0).fill(u8::MAX);

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        source.broadcast_video(&frame, timebase).await?;

        tracing::info!(
            "Currently connected peers: {} (tally: {:?})",
            source.peers().await.len(),
            source.tally().await
        );
    }
}
