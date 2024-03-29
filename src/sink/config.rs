use crate::io::frame::text;

#[cfg(doc)]
use super::Sink;

/// Configuration for the [`Sink`] structure.
#[derive(Debug, Default, Clone)]
pub struct Config<'s> {
    /// Sink name to advertise over the network, defaults to `receiver`.
    pub name: Option<&'s str>,

    /// Size of the [`ffmpeg::frame::Video`] queue to be retained until incoming frames are dropped. Set to `0` to disable video streaming.
    pub video_queue: usize,

    /// Size of the [`ffmpeg::frame::Audio`] queue to be retained until incoming frames are dropped. Set to `0` to disable audio streaming.
    pub audio_queue: usize,

    /// Quality of the video stream to request to the source.
    pub video_quality: text::VideoQuality,
}
