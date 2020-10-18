#![cfg(feature = "io-tokio")]

use crate::{
    AudioDataHeader, Error, Header, MetaData, Result, TagHeader, TagType, VideoDataHeader,
};
use core::convert::TryFrom;
use std::io::SeekFrom;
use tokio::prelude::io::*;

pub struct FlvWriter<W> {
    writer: W,
}

impl<W: AsyncWrite + Unpin> FlvWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub async fn write_header(&mut self, header: Header) -> Result<u64> {
        let buffer: [u8; 9] = header.into();
        self.writer.write_all(&buffer).await?;

        // PreviousTagSize0 is 0u32
        self.writer.write_all(&[0, 0, 0, 0]).await?;

        Ok(9 + 4)
    }

    pub async fn write_metadata(&mut self, _metadata: MetaData) -> Result<u64> {
        Ok(0)
    }

    async fn write_tag(
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

        self.writer.write_all(&th_data).await?;
        self.writer.write_all(header).await?;
        self.writer.write_all(data).await?;

        Ok((TagHeader::SIZE + 1 + data_size) as u64)
    }

    pub async fn write_video_tag(
        &mut self,
        timestamp: i32,
        header: VideoDataHeader,
        data: &[u8],
    ) -> Result<u64> {
        self.write_tag(timestamp, TagType::Video, &[u8::from(header)], data)
            .await
    }

    pub async fn write_audio_tag(
        &mut self,
        timestamp: i32,
        header: AudioDataHeader,
        data: &[u8],
    ) -> Result<u64> {
        self.write_tag(timestamp, TagType::Audio, &[u8::from(header)], data)
            .await
    }
}

pub struct FlvReader<R> {
    reader: R,
}

impl<R: AsyncRead + AsyncSeek + Unpin> FlvReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub async fn read_header(&mut self) -> Result<Header> {
        let mut buffer = [0u8; Header::SIZE];
        self.reader.read_exact(&mut buffer).await?;

        Ok(Header::try_from(&buffer)?)
    }

    pub async fn read_metadata(&mut self) -> Result<MetaData> {
        unimplemented!()
    }

    pub async fn read_tag_header(&mut self) -> Result<TagHeader> {
        let mut buffer = [0u8; TagHeader::SIZE];
        self.reader.read_exact(&mut buffer).await?;

        Ok((&buffer).into())
    }

    pub async fn read_video_data_header(&mut self) -> Result<VideoDataHeader> {
        let mut buffer = [0u8; 1];
        self.reader.read_exact(&mut buffer).await?;

        Ok(VideoDataHeader::try_from(buffer[0])?)
    }

    pub async fn read_audio_data_header(&mut self) -> Result<AudioDataHeader> {
        let mut buffer = [0u8; 1];
        self.reader.read_exact(&mut buffer).await?;

        Ok(AudioDataHeader::try_from(buffer[0])?)
    }
}
