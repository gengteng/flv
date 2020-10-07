#![cfg(feature = "io-std")]

use crate::{
    AudioDataHeader, Error, Header, MetaData, Result, TagHeader, TagType, VideoDataHeader,
};
use std::convert::TryFrom;
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(Debug)]
pub struct Tag<D> {
    pub header: TagHeader,
    pub data: TagData<D>,
}

#[derive(Debug)]
pub struct AudioData<D> {
    pub header: AudioDataHeader,
    pub data: D,
}

#[derive(Debug)]
pub struct VideoData<D> {
    pub header: VideoDataHeader,
    pub data: D,
}

#[derive(Debug)]
pub struct ScriptData<D> {
    pub data: D,
}

#[derive(Debug)]
pub enum TagData<D> {
    Audio(AudioData<D>),
    Video(VideoData<D>),
    ScriptData(ScriptData<D>),
    Reserved(D),
}

#[derive(Debug)]
pub enum Field<D> {
    PreTagSize(u32),
    Tag(Tag<D>),
}

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

pub struct FlvReader<R> {
    reader: R,
}

impl<R: Read + Seek> FlvReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn read_header(&mut self) -> Result<Header> {
        self.reader.seek(SeekFrom::Start(0))?;

        let mut buffer = [0u8; Header::SIZE];
        self.reader.read_exact(&mut buffer)?;

        Ok(Header::try_from(buffer)?)
    }

    pub fn read_metadata(&mut self) -> Result<MetaData> {
        unimplemented!()
    }

    pub fn read_tag_header(&mut self) -> Result<TagHeader> {
        let mut buffer = [0u8; TagHeader::SIZE];
        self.reader.read_exact(&mut buffer)?;

        Ok(buffer.into())
    }

    pub fn read_video_data_header(&mut self) -> Result<VideoDataHeader> {
        let mut buffer = [0u8; 1];
        self.reader.read_exact(&mut buffer)?;

        Ok(VideoDataHeader::try_from(buffer[0])?)
    }

    pub fn read_audio_data_header(&mut self) -> Result<AudioDataHeader> {
        let mut buffer = [0u8; 1];
        self.reader.read_exact(&mut buffer)?;

        Ok(AudioDataHeader::try_from(buffer[0])?)
    }
}
