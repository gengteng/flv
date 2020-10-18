#![cfg(feature = "io-std")]

use crate::error::ReadError;
pub use crate::io::*;
use crate::{
    AudioDataHeader, Error, Header, MetaData, Result, TagHeader, TagType, VideoDataHeader,
};
use core::convert::TryFrom;
use std::collections::BTreeMap;
use std::io::{ErrorKind, Read, Write};

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

impl<R: Read> FlvReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn read_header(&mut self) -> Result<Header> {
        let mut buffer = [0u8; Header::SIZE];
        self.try_read_exact(&mut buffer)?;

        Ok(Header::try_from(&buffer)?)
    }

    pub fn read_metadata(&mut self) -> Result<MetaData> {
        let mut metadata = MetaData::default();

        let marker = {
            let mut marker = [0u8; 1];
            self.reader.read_exact(&mut marker)?;
            marker[0]
        };

        if marker != 0x02 {
            return Err(Error::Other("marker error"));
        }

        let len = {
            let mut len = [0u8; 2];
            self.reader.read_exact(&mut len)?;
            u16::from_be_bytes(len)
        } as usize;

        let name = {
            let mut name = vec![0u8; len];
            self.reader.read_exact(&mut name)?;
            String::from_utf8(name)?
        };

        if name != "onMetaData" {
            return Err(Error::Other("invalid onMetaData"));
        }

        // ECMA Array
        let marker = {
            let mut marker = [0u8; 1];
            self.reader.read_exact(&mut marker)?;
            marker[0]
        };

        if marker != 0x08 {
            return Err(Error::Other("marker error"));
        }

        let array_len = {
            let mut array_len = [0u8; 4];
            self.reader.read_exact(&mut array_len)?;
            u32::from_be_bytes(array_len)
        } as usize;

        for _ in 0..array_len {
            let len = {
                let mut len = [0u8; 2];
                self.reader.read_exact(&mut len)?;
                u16::from_be_bytes(len)
            } as usize;

            let key = {
                let mut key = vec![0u8; len];
                self.reader.read_exact(&mut key)?;
                String::from_utf8(key)?
            };

            let marker = {
                let mut marker = [0u8; 1];
                self.reader.read_exact(&mut marker)?;
                marker[0]
            };

            match marker {
                0 => {
                    // double
                    let value = {
                        let mut value = [0u8; 8];
                        self.reader.read_exact(&mut value)?;
                        f64::from_be_bytes(value)
                    };

                    match key.as_str() {
                        "duration" => metadata.duration = value,
                        "width" => metadata.width = value,
                        "height" => metadata.height = value,
                        "videodatarate" => metadata.video_data_rate = value,
                        "framerate" => metadata.framerate = value,
                        "videocodecid" => metadata.video_codec_id = value,
                        "audiodatarate" => metadata.audio_date_rate = value,
                        "audiosamplerate" => metadata.audio_sample_rate = value,
                        "audiosamplesize" => metadata.audio_sample_size = value,
                        "audiocodecid" => metadata.audio_codec_id = value,
                        "filesize" => metadata.filesize = value,
                        "datasize" => metadata.data_size = value,
                        "videosize" => metadata.video_size = value,
                        "audiosize" => metadata.audio_size = value,
                        "lasttimestamp" => metadata.last_timestamp = value,
                        "lastkeyframetimestamp" => metadata.last_keyframe_timestamp = value,
                        "lastkeyframelocation" => metadata.last_keyframe_location = value,
                        _ => {}
                    }
                }
                1 => {
                    // bool
                    let value = {
                        let mut value = [0u8; 1];
                        self.reader.read_exact(&mut value)?;
                        value[0]
                    } != 0;

                    match key.as_str() {
                        "stereo" => metadata.stereo = value,
                        "hasVideo" => metadata.has_video = value,
                        "hasKeyframes" => metadata.has_keyframes = value,
                        "hasAudio" => metadata.has_audio = value,
                        "hasMetadata" => metadata.has_metadata = value,
                        "canSeekToEnd" => metadata.can_seek_to_end = value,
                        _ => {}
                    }
                }
                2 => {
                    // string
                    let len = {
                        let mut len = [0u8; 2];
                        self.reader.read_exact(&mut len)?;
                        u16::from_be_bytes(len)
                    } as usize;

                    let value = {
                        let mut value = vec![0u8; len];
                        self.reader.read_exact(&mut value)?;
                        String::from_utf8(value)?
                    };

                    match key.as_str() {
                        "major_brand" => metadata.major_brand = value,
                        "minor_version" => metadata.minor_version = value,
                        "compatible_brands" => metadata.compatible_brands = value,
                        "encoder" => metadata.encoder = value,
                        _ => {}
                    }
                }
                3 if key == "keyframes" => {
                    //script data object array

                    let len = {
                        let mut len = [0u8; 2];
                        self.reader.read_exact(&mut len)?;
                        u16::from_be_bytes(len)
                    } as usize;

                    let key = {
                        let mut key = vec![0u8; len];
                        self.reader.read_exact(&mut key)?;
                        String::from_utf8(key)?
                    };

                    if key != "filepositions" {
                        return Err(Error::Other("invalid filepositions key"));
                    }

                    let marker = {
                        let mut marker = [0u8; 1];
                        self.reader.read_exact(&mut marker)?;
                        marker[0]
                    };

                    if marker != 0x0a {
                        return Err(Error::Other("invalid filepositions marker"));
                    }

                    let len = {
                        let mut len = [0u8; 4];
                        self.reader.read_exact(&mut len)?;
                        u32::from_be_bytes(len)
                    } as usize;

                    let mut positions = Vec::with_capacity(len);

                    for _ in 0..len {
                        let marker = {
                            let mut marker = [0u8; 1];
                            self.reader.read_exact(&mut marker)?;
                            marker[0]
                        };

                        if marker != 0 {
                            return Err(Error::Other("invalid filepositions item marker"));
                        }

                        positions.push({
                            let mut value = [0u8; 8];
                            self.reader.read_exact(&mut value)?;
                            f64::from_be_bytes(value)
                        } as u64);
                    }

                    let len = {
                        let mut len = [0u8; 2];
                        self.reader.read_exact(&mut len)?;
                        u16::from_be_bytes(len)
                    } as usize;

                    let key = {
                        let mut key = vec![0u8; len];
                        self.reader.read_exact(&mut key)?;
                        String::from_utf8(key)?
                    };

                    if key != "times" {
                        return Err(Error::Other("invalid times key"));
                    }

                    let marker = {
                        let mut marker = [0u8; 1];
                        self.reader.read_exact(&mut marker)?;
                        marker[0]
                    };

                    if marker != 0x0a {
                        return Err(Error::Other("invalid times marker"));
                    }

                    let len = {
                        let mut len = [0u8; 4];
                        self.reader.read_exact(&mut len)?;
                        u32::from_be_bytes(len)
                    } as usize;

                    let mut times = Vec::with_capacity(len);

                    for _ in 0..len {
                        let marker = {
                            let mut marker = [0u8; 1];
                            self.reader.read_exact(&mut marker)?;
                            marker[0]
                        };

                        if marker != 0 {
                            return Err(Error::Other("invalid times item marker"));
                        }

                        times.push(
                            ({
                                let mut value = [0u8; 8];
                                self.reader.read_exact(&mut value)?;
                                f64::from_be_bytes(value)
                            } * 1000.0) as u32,
                        );
                    }

                    let map = times
                        .drain(..)
                        .zip(positions.drain(..))
                        .collect::<BTreeMap<_, _>>();

                    metadata.keyframes = Some(map);

                    self.read_end_marker()?;
                }
                n => {
                    return Err(Error::Unimplemented(format!(
                        "unimplemented script object type: {}",
                        n
                    )))
                }
            }
        }

        self.read_end_marker()?;
        Ok(metadata)
    }

    pub fn read_end_marker(&mut self) -> Result<()> {
        let end = {
            let mut end = [0u8; 3];
            self.reader.read_exact(&mut end)?;
            u32::from_be_bytes([0, end[0], end[1], end[2]])
        };

        if end != 9 {
            return Err(Error::Other("invalid end of object"));
        }

        Ok(())
    }

    pub fn read_pre_tag_size(&mut self) -> Result<u32> {
        let mut buffer = [0u8; 4];
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

        Ok(Some((&buffer).into()))
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

    pub fn read_data(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; len];
        self.try_read_exact(&mut buffer[..len])?;
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
