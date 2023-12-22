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

    tracing::info!("Got source: {source:?}");

    let recv = Recv::new(&source, 16)?;

    tracing::info!("Connected !");

    for video in recv.video_frames() {
        let video = video?;

        let mut converted = ffmpeg_next::frame::Video::empty();
        video
            .converter(ffmpeg_next::format::Pixel::RGBA)?
            .run(&video, &mut converted)?;

        tracing::warn!(
            "{:?}, {}px x {}px",
            converted.format(),
            converted.width(),
            converted.height(),
        );
    }

    Ok(())
}
