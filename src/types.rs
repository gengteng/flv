use crate::error::ParseError;
use core::convert::TryFrom;
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// FLV file header
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Header {
    /// File version (for example, 0x01 for FLV version 1)
    pub version: u8,

    /// audio tags are present
    pub audio_flag: bool,

    /// video tags are present
    pub video_flag: bool,

    /// The DataOffset field usually has a value of 9 for FLV version 1.
    /// This field is present to accommodate larger headers in future versions.
    pub data_offset: u32,
}

impl Header {
    pub const SIGNATURE: [u8; 3] = [b'F', b'L', b'V'];
    pub const VERSION_1: u8 = 0x01;
    pub const SIZE: usize = 9;
}

impl TryFrom<&[u8; Header::SIZE]> for Header {
    type Error = ParseError;

    fn try_from(value: &[u8; Header::SIZE]) -> Result<Self, ParseError> {
        let [f, l, v, version, flag, d1, d2, d3, d4] = *value;

        match [f, l, v] {
            Self::SIGNATURE => {}
            [s1, s2, s3] => return Err(ParseError::HeaderSignature(s1, s2, s3)),
        }

        if version != Self::VERSION_1 {
            return Err(ParseError::HeaderVersion(version));
        }

        let reserved_flag = 0b11111010 & flag;
        if reserved_flag != 0 {
            return Err(ParseError::HeaderTypeFlagsReserved(reserved_flag));
        }

        let audio_flag = 0b00000100 & flag != 0;
        let video_flag = 0b00000001 & flag != 0;
        let data_offset = u32::from_be_bytes([d1, d2, d3, d4]);

        if data_offset != Self::SIZE as u32 {
            return Err(ParseError::HeaderDataOffset(data_offset));
        }

        Ok(Self {
            version,
            audio_flag,
            video_flag,
            data_offset,
        })
    }
}

impl From<Header> for [u8; Header::SIZE] {
    fn from(h: Header) -> Self {
        let flag =
            if h.audio_flag { 0b0000100 } else { 0 } | if h.video_flag { 0b00000001 } else { 0 };

        let [o1, o2, o3, o4] = (Header::SIZE as u32).to_be_bytes();

        [
            Header::SIGNATURE[0],
            Header::SIGNATURE[1],
            Header::SIGNATURE[2],
            h.version,
            flag,
            o1,
            o2,
            o3,
            o4,
        ]
    }
}

#[test]
fn parse_header() {
    let header = Header {
        version: 1,
        audio_flag: true,
        video_flag: true,
        data_offset: 9,
    };

    let bytes: [u8; Header::SIZE] = header.into();

    let parsed = Header::try_from(&bytes);

    assert_eq!(Ok(header), parsed);
    assert_eq!(Ok(bytes), parsed.map(|h| h.into()));
}

/// Flv tag type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagType {
    Audio,        // 8
    Video,        // 9
    ScriptData,   // 18
    Reserved(u8), // all others
}

impl From<TagType> for u8 {
    fn from(tt: TagType) -> Self {
        match tt {
            TagType::Audio => 8,
            TagType::Video => 9,
            TagType::ScriptData => 18,
            TagType::Reserved(n) => n,
        }
    }
}

/// Flv tag header
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TagHeader {
    pub tag_type: TagType,
    pub data_size: u32,
    pub timestamp: i32, // UI24 + UI8 => SI32
                        // pub stream_id: u32, // UI24 always 0
}

impl TagHeader {
    pub const SIZE: usize = (8 + 24 + 24 + 8 + 24) / 8;
    pub const MAX_DATA_SIZE: usize = 0x00ffffff; // u24::max_value()
}

impl From<&[u8; TagHeader::SIZE]> for TagHeader {
    fn from(value: &[u8; TagHeader::SIZE]) -> Self {
        let [tt, s1, s2, s3, t1, t2, t3, t0, _, _, _] = *value;
        let tag_type = match tt {
            8 => TagType::Audio,
            9 => TagType::Video,
            18 => TagType::ScriptData,
            n => TagType::Reserved(n),
        };

        // UI24 big endian
        let data_size = u32::from_be_bytes([0, s1, s2, s3]);

        // t0: Extension of the timestamp field to form a SI32 value.
        // This field represents the upper 8 bits, while the previous timestamp
        // field represents the lower 24 bits of the time in milliseconds.
        //
        // t1~t3: time in milliseconds which the data in this tag applies.
        // This value is relative to the first tag in the FLV file, which always
        // has a timestamp of 0.
        let timestamp = i32::from_be_bytes([t0, t1, t2, t3]);

        TagHeader {
            tag_type,
            data_size,
            timestamp,
        }
    }
}

impl From<TagHeader> for [u8; TagHeader::SIZE] {
    fn from(h: TagHeader) -> Self {
        let tt = h.tag_type.into();

        let [_, s1, s2, s3] = h.data_size.to_be_bytes();
        let [t0, t1, t2, t3] = h.timestamp.to_be_bytes();

        [tt, s1, s2, s3, t1, t2, t3, t0, 0, 0, 0]
    }
}

/// Sound format
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SoundFormat {
    LinearPCMPlatformEndian = 0,
    ADPCM = 1,
    MP3 = 2,
    LinearPCMLittleEndian = 3,
    Nellymoser16 = 4,
    Nellymoser8 = 5,
    Nellymoser = 6,
    G711ALaw = 7,
    G711MuLaw = 8,
    Reserved = 9,
    AAC = 10,
    Speex = 11,
    MP38kHz = 14,
    DeviceSpecific = 15,
}

impl TryFrom<u8> for SoundFormat {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use SoundFormat::*;
        Ok(match value {
            0 => LinearPCMPlatformEndian,
            1 => ADPCM,
            2 => MP3,
            3 => LinearPCMLittleEndian,
            4 => Nellymoser16,
            5 => Nellymoser8,
            6 => Nellymoser,
            7 => G711ALaw,
            8 => G711MuLaw,
            9 => Reserved,
            10 => AAC,
            11 => Speex,
            14 => MP38kHz,
            15 => DeviceSpecific,
            n => return Err(ParseError::SoundFormat(n)),
        })
    }
}

impl From<SoundFormat> for u8 {
    fn from(sf: SoundFormat) -> Self {
        sf as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SoundRate {
    R5p5kHz = 0,
    R11kHz = 1,
    R22kHz = 2,
    R44kHz = 3,
}

impl TryFrom<u8> for SoundRate {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use SoundRate::*;
        Ok(match value {
            0 => R5p5kHz,
            1 => R11kHz,
            2 => R22kHz,
            3 => R44kHz,
            n => return Err(ParseError::SoundRate(n)),
        })
    }
}

impl From<SoundRate> for u8 {
    fn from(sr: SoundRate) -> Self {
        sr as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SoundSize {
    S8Bit = 0,
    S16Bit = 1,
}

impl TryFrom<u8> for SoundSize {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use SoundSize::*;
        Ok(match value {
            0 => S8Bit,
            1 => S16Bit,
            n => return Err(ParseError::SoundSize(n)),
        })
    }
}

impl From<SoundSize> for u8 {
    fn from(ss: SoundSize) -> Self {
        ss as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SoundType {
    Mono = 0,
    Stereo = 1,
}

impl TryFrom<u8> for SoundType {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use SoundType::*;
        Ok(match value {
            0 => Mono,
            1 => Stereo,
            n => return Err(ParseError::SoundType(n)),
        })
    }
}

impl From<SoundType> for u8 {
    fn from(st: SoundType) -> Self {
        st as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioDataHeader {
    pub sound_format: SoundFormat,
    pub sound_rate: SoundRate,
    pub sound_size: SoundSize,
    pub sound_type: SoundType,
}

impl TryFrom<u8> for AudioDataHeader {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let sound_format = SoundFormat::try_from((value & 0b_1111_0000) >> 4)?;
        let sound_rate = SoundRate::try_from((value & 0b_0000_1100) >> 2)?;
        let sound_size = SoundSize::try_from((value & 0b_0000_0010) >> 1)?;
        let sound_type = SoundType::try_from(value & 0b_0000_0001)?;

        Ok(Self {
            sound_format,
            sound_rate,
            sound_size,
            sound_type,
        })
    }
}

impl From<AudioDataHeader> for u8 {
    fn from(h: AudioDataHeader) -> Self {
        (u8::from(h.sound_format) << 4)
            | (u8::from(h.sound_rate) << 2)
            | (u8::from(h.sound_size) << 1)
            | u8::from(h.sound_type)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VideoFrameType {
    KeyFrame = 1,
    InterFrame = 2,
    DisposableInterFrame = 3,
    GeneratedKeyFrame = 4,
    VideoInfoOrCommandFrame = 5,
}

impl TryFrom<u8> for VideoFrameType {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use VideoFrameType::*;
        Ok(match value {
            1 => KeyFrame,
            2 => InterFrame,
            3 => DisposableInterFrame,
            4 => GeneratedKeyFrame,
            5 => VideoInfoOrCommandFrame,
            n => return Err(ParseError::VideoFrameType(n)),
        })
    }
}

impl From<VideoFrameType> for u8 {
    fn from(vft: VideoFrameType) -> Self {
        vft as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VideoCodecId {
    JPEG = 1,
    SorensonH263 = 2,
    ScreenVideo = 3,
    On2VP6 = 4,
    On2VP6WithAlpha = 5,
    ScreenVideoVersion2 = 6,
    AVC = 7,
    // AV1 = 13,
}

impl TryFrom<u8> for VideoCodecId {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use VideoCodecId::*;
        Ok(match value {
            1 => JPEG,
            2 => SorensonH263,
            3 => ScreenVideo,
            4 => On2VP6,
            5 => On2VP6WithAlpha,
            6 => ScreenVideoVersion2,
            7 => AVC,
            // 13 => AV1,
            n => return Err(ParseError::VideoCodecId(n)),
        })
    }
}

impl From<VideoCodecId> for u8 {
    fn from(vci: VideoCodecId) -> Self {
        vci as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VideoDataHeader {
    pub frame_type: VideoFrameType,
    pub codec_id: VideoCodecId,
}

impl TryFrom<u8> for VideoDataHeader {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let frame_type = VideoFrameType::try_from(value >> 4)?;
        let codec_id = VideoCodecId::try_from(value & 0xf)?;

        Ok(Self {
            frame_type,
            codec_id,
        })
    }
}

impl From<VideoDataHeader> for u8 {
    fn from(h: VideoDataHeader) -> Self {
        (u8::from(h.frame_type) << 4) | u8::from(h.codec_id)
    }
}

/// SeekFlag for client-side seeking video frame sequence
///
/// if FrameType = 5, instead of a video payload, the message stream contains
/// a UI8 with the following meaning:
/// * 0 = Start of client-side seeking video frame sequence
/// * 1 = End of client-side seeking video frame sequence
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SeekFlag {
    Start = 0,
    End = 1,
}

impl TryFrom<u8> for SeekFlag {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use SeekFlag::*;

        Ok(match value {
            0 => Start,
            1 => End,
            n => return Err(ParseError::SeekFlag(n)),
        })
    }
}

impl From<SeekFlag> for u8 {
    fn from(sf: SeekFlag) -> Self {
        sf as u8
    }
}

#[derive(Debug, Default, Clone)]
pub struct MetaData {
    pub duration: f64,
    pub width: f64,
    pub height: f64,
    pub video_data_rate: f64,
    pub framerate: f64,
    pub video_codec_id: f64,
    pub audio_date_rate: f64,
    pub audio_sample_rate: f64,
    pub audio_sample_size: f64,
    pub stereo: bool,
    pub audio_codec_id: f64,
    pub major_brand: String,
    pub minor_version: String,
    pub compatible_brands: String,
    pub encoder: String,
    pub filesize: f64,
    pub has_video: bool,
    pub has_keyframes: bool,
    pub has_audio: bool,
    pub has_metadata: bool,
    pub can_seek_to_end: bool,
    pub data_size: f64,
    pub video_size: f64,
    pub audio_size: f64,
    pub last_timestamp: f64,
    pub last_keyframe_timestamp: f64,
    pub last_keyframe_location: f64,
    pub keyframes: Option<BTreeMap<u32, u64>>,
}

impl MetaData {
    pub fn seek(&self, timestamp: u32) -> Option<(u32, u64)> {
        let mut target = None;
        if let Some(keyframes) = &self.keyframes {
            for (ts, offset) in keyframes {
                match ts.cmp(&timestamp) {
                    Ordering::Less => target = Some((*ts, *offset)),
                    Ordering::Greater => break,
                    Ordering::Equal => return Some((timestamp, *offset)),
                }
            }
            target
        } else {
            None
        }
    }
}
