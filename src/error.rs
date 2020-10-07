use thiserror::Error as ThisError;

/// A `Result` typedef to use with the `flv::Error` type
pub type Result<T> = std::result::Result<T, Error>;

/// flv error type
#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[cfg(feature = "io-std")]
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("data size is too long: {0}")]
    DataSize(usize),
}

/// parse error
#[derive(ThisError, Debug, Eq, PartialEq)]
pub enum ParseError {
    #[error("invalid header signature: 0x{0:X}, 0x{1:X}, 0x{2:X}")]
    HeaderSignature(u8, u8, u8),
    #[error("invalid reserved type flags format: 0x{0:X}")]
    HeaderTypeFlagsReserved(u8),
    #[error("invalid sound format: 0x{0:X}")]
    SoundFormat(u8),
    #[error("invalid sound rate: 0x{0:X}")]
    SoundRate(u8),
    #[error("invalid sound size: 0x{0:X}")]
    SoundSize(u8),
    #[error("invalid sound type: 0x{0:X}")]
    SoundType(u8),
    #[error("invalid video frame type: 0x{0:X}")]
    VideoFrameType(u8),
    #[error("invalid video codec id: 0x{0:X}")]
    VideoCodecId(u8),
    #[error("invalid seek flag: 0x{0:X}")]
    SeekFlag(u8),
}
