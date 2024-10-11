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

    let timebase = ffmpeg::sys::AVRational { num: 1, den: 30 };
    let mut frame = ffmpeg::frame::Video::new(ffmpeg::format::Pixel::RGB24, 1920, 1080);

    let mut idx = 0u8;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(
            (1000 * timebase.num / timebase.den) as u64,
        ));

        for pix in frame.plane_mut(0) {
            *pix = (idx, idx.wrapping_mul(2), idx.wrapping_mul(3));

            idx = idx.wrapping_add(1);
        }
        source.broadcast_video(&frame, timebase).await?;

        idx = idx.wrapping_add(1);

        tracing::info!(
            "Currently connected peers: {} (tally: {:?})",
            source.peers().await.len(),
            source.tally().await
        );
    }
}
