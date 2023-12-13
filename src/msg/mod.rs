mod video;
pub use video::Video;

mod audio;
pub use audio::Audio;

mod text;
pub use text::{info, Text};

#[derive(Debug)]
pub enum Msg {
    Video(Video),
    Audio(Audio),
    Text(Text),
}
