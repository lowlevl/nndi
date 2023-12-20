use nndi::{recv::Recv, scan::Scan};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    for video in recv.iter_video() {
        tracing::warn!("{:?}", video?);
    }

    Ok(())
}
