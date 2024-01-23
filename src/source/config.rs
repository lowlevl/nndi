#[cfg(doc)]
use super::Source;

/// Configuration for the [`Source`] structure.
#[derive(Debug, Default, Clone)]
pub struct Config {
    /// Source name to advertise over the network.
    pub name: String,

    /// Source groups to advertise over the network, defaults to `public`.
    pub groups: Option<Vec<&'static str>>,
}
