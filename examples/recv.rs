use nndi::{Scan, Sink};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let sink = Sink::new(
        &source,
        nndi::sink::Config {
            video_queue: 4,
            audio_queue: 4,
            ..Default::default()
        },
    )
    .await?;

    tracing::info!("Connected to source: {source:?}");

    let t0 = std::thread::spawn({
        let sink = sink.clone();

        move || {
            for (idx, video) in sink.video_frames().enumerate() {
                let video = video.expect("Unable to decode `video` frame");

                tracing::warn!(
                    "#{idx}: {:?}, {}px x {}px",
                    video.format(),
                    video.width(),
                    video.height(),
                );
            }
        }
    });

    let t1 = std::thread::spawn(move || {
        for (idx, audio) in sink.audio_frames().enumerate() {
            let audio = audio.expect("Unable to decode `audio` frame");

            tracing::warn!(
                "#{idx}: {:?}, rate: {} samples: {}",
                audio.format(),
                audio.rate(),
                audio.samples(),
            );
        }
    });

    t0.join().expect("Unnable to join `video` thread");
    t1.join().expect("Unnable to join `audio` thread");

    Ok(())
}
