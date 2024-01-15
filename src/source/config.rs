#[cfg(doc)]
use super::Source;

/// Configuration for the [`Source`] structure.
#[derive(Debug, Default, Clone)]
pub struct Config<'s> {
    /// Source name to advertise over the network.
    pub name: &'s str,

    /// Source groups to advertise over the network, defaults to `public`.
    pub groups: Option<&'s [&'s str]>,
}
