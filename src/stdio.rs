#![cfg(feature = "io-std")]

use crate::error::ReadError;
pub use crate::io::*;
use crate::{
    AudioDataHeader, Error, Header, MetaData, ParseError, Result, TagHeader, TagType,
    VideoDataHeader,
};
use amf::amf0::Value;
use amf::Pair;
use std::convert::TryFrom;
use std::io::{ErrorKind, Read, Seek, Write};
use std::mem::size_of;

pub struct FlvWriter<W> {
    writer: W,
}

impl<W: Write> FlvWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn write_header(&mut self, header: Header) -> Result<u64> {
        let buffer: [u8; 9] = header.into();
        self.writer.write_all(&buffer)?;

        // PreviousTagSize0 is 0u32
        self.writer.write_all(&[0, 0, 0, 0])?;

        Ok(9 + 4)
    }

    pub fn write_metadata(&mut self, _metadata: MetaData) -> Result<u64> {
        Ok(0)
    }

    fn write_tag(
        &mut self,
        timestamp: i32,
        tag_type: TagType,
        header: &[u8],
        data: &[u8],
    ) -> Result<u64> {
        let data_size = data.len();

        if data_size > TagHeader::MAX_DATA_SIZE {
            return Err(Error::DataSize(data_size));
        }

        let tag_header = TagHeader {
            tag_type,
            data_size: data_size as u32,
            timestamp,
        };

        let th_data: [u8; TagHeader::SIZE] = tag_header.into();

        self.writer.write_all(&th_data)?;
        self.writer.write_all(header)?;
        self.writer.write_all(data)?;

        Ok((TagHeader::SIZE + 1 + data_size) as u64)
    }

    pub fn write_video_tag(
        &mut self,
        timestamp: i32,
        header: VideoDataHeader,
        data: &[u8],
    ) -> Result<u64> {
        self.write_tag(timestamp, TagType::Video, &[u8::from(header)], data)
    }

    pub fn write_audio_tag(
        &mut self,
        timestamp: i32,
        header: AudioDataHeader,
        data: &[u8],
    ) -> Result<u64> {
        self.write_tag(timestamp, TagType::Audio, &[u8::from(header)], data)
    }
}

pub struct FlvReader<R, C> {
    reader: R,
    cache: C,
}

impl<R: Read + Seek, C: IndexCache> FlvReader<R, C> {
    pub fn new(reader: R, cache: C) -> Self {
        Self { reader, cache }
    }

    pub fn read_header(&mut self) -> Result<Header> {
        let mut buffer = [0u8; Header::SIZE];
        self.try_read_exact(&mut buffer)?;

        Ok(Header::try_from(buffer)?)
    }

    pub fn read_metadata(&mut self) -> Result<MetaData> {
        let _name = amf::Amf0Value::read_from(&mut self.reader).unwrap();
        let value = amf::Amf0Value::read_from(&mut self.reader).unwrap();

        let mut metadata = MetaData::default();

        match value {
            Value::EcmaArray { mut entries } => {
                for Pair { key, value } in entries.drain(..) {
                    match (key.as_str(), value) {
                        ("duration", Value::Number(duration)) => metadata.duration = duration,
                        ("width", Value::Number(width)) => metadata.width = width,
                        ("height", Value::Number(height)) => metadata.height = height,
                        ("videodatarate", Value::Number(videodatarate)) => {
                            metadata.videodatarate = videodatarate
                        }
                        ("framerate", Value::Number(framerate)) => metadata.framerate = framerate,
                        ("videocodecid", Value::Number(videocodecid)) => {
                            metadata.videocodecid = videocodecid
                        }
                        ("audiosamplerate", Value::Number(audiosamplerate)) => {
                            metadata.audiosamplerate = audiosamplerate
                        }
                        ("audiosamplesize", Value::Number(audiosamplesize)) => {
                            metadata.audiosamplesize = audiosamplesize
                        }
                        ("stereo", Value::Boolean(stereo)) => metadata.stereo = stereo,
                        ("audiocodecid", Value::Number(audiocodecid)) => {
                            metadata.audiocodecid = audiocodecid
                        }
                        ("filesize", Value::Number(filesize)) => metadata.filesize = filesize,
                        _ => (),
                    }
                }
            }
            _ => {
                return Err(Error::Parse(ParseError::MetadataType));
            }
        }

        Ok(metadata)
    }

    pub fn read_pre_tag_size(&mut self) -> Result<u32> {
        let mut buffer = [0u8; size_of::<u32>()];
        self.try_read_exact(&mut buffer)?;

        Ok(u32::from_be_bytes(buffer))
    }

    pub fn read_tag_header(&mut self) -> Result<Option<TagHeader>> {
        let mut buffer = [0u8; TagHeader::SIZE];
        if let Err(e) = self.try_read_exact(&mut buffer) {
            return match e {
                Error::Read(ReadError::Eof) => Ok(None),
                e => Err(e),
            };
        }

        Ok(Some(buffer.into()))
    }

    pub fn read_video_data_header(&mut self) -> Result<VideoDataHeader> {
        let mut buffer = [0u8; 1];
        self.try_read_exact(&mut buffer)?;

        Ok(VideoDataHeader::try_from(buffer[0])?)
    }

    pub fn read_audio_data_header(&mut self) -> Result<AudioDataHeader> {
        let mut buffer = [0u8; 1];
        self.try_read_exact(&mut buffer)?;

        Ok(AudioDataHeader::try_from(buffer[0])?)
    }

    pub fn read_data(&mut self, TagHeader { data_size, .. }: TagHeader) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; data_size as usize - 1];
        self.try_read_exact(&mut buffer[..data_size as usize - 1])?;
        Ok(buffer)
    }

    fn try_read_exact(&mut self, mut buf: &mut [u8]) -> Result<()> {
        let len = buf.len();
        while !buf.is_empty() {
            match self.reader.read(buf) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(Error::Io(e)),
            }
        }
        if !buf.is_empty() {
            if len == buf.len() {
                Err(Error::Read(ReadError::Eof))
            } else {
                Err(Error::Io(std::io::Error::new(
                    ErrorKind::UnexpectedEof,
                    "failed to fill whole buffer",
                )))
            }
        } else {
            Ok(())
        }
    }
}
