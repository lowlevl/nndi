use nndi::send::Send;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set-up the log and traces handler
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let _send = Send::new("super source", None)?;

    loop {}
}
