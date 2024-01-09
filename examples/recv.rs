use nndi::{recv::Recv, scan::Scan};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg_next::init()?;

    // Set-up the log and traces handler
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let mut scan = Scan::new()?;

    let source = loop {
        let sources = scan.sources().collect::<Vec<_>>();

        std::thread::sleep(std::time::Duration::from_millis(100));

        let Some(source) = sources.get(0) else {
            continue;
        };

        break (*source).clone();
    };

    let recv = Recv::new(&source, 16)?;

    tracing::info!("Connected to source: {source:?}");

    for (idx, video) in recv.video_frames().enumerate() {
        let video = video?;

        tracing::warn!(
            "#{idx}: {:?}, {}px x {}px",
            video.format(),
            video.width(),
            video.height(),
        );
    }

    Ok(())
}
