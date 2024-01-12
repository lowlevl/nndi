use nndi::{ffmpeg, Source};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set-up the log and traces handler
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let send = Source::new("super source", None)?;

    let timebase = ffmpeg::sys::AVRational { num: 1, den: 1 };
    let mut frame = ffmpeg::frame::Video::new(ffmpeg::format::Pixel::RGBA, 600, 360);
    frame.data_mut(0).fill(u8::MAX);

    let mut idx = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        frame.set_pts(Some(idx));
        send.send_video(&frame, timebase)?;

        idx += 1;
    }
}
