mod error;
mod types;

pub use crate::error::{Error, ParseError, Result};
pub use crate::types::{
    AudioDataHeader, Header, MetaData, SeekFlag, SoundFormat, SoundRate, SoundSize, SoundType,
    TagHeader, TagType, VideoCodecId, VideoDataHeader, VideoFrameType,
};

#[macro_use]
mod cfg;

cfg_io_tokio! {
    pub mod tokio;
}

cfg_io_std! {
    pub mod io;
}
