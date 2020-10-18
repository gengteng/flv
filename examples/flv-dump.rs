use flv::stdio::FlvReader;
use flv::TagType;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

/// flv dump
#[derive(Debug, StructOpt)]
#[structopt(name = "flv-dump", about = "A flv file dump tool.")]
struct Opts {
    /// flv file path
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::from_args();

    let file = File::open(opts.file)?;

    let mut reader = FlvReader::new(file);
    println!("flv header: {:?}", reader.read_header()?);

    let mut index = 0usize;

    while let Ok(pre_tag_size) = reader.read_pre_tag_size() {
        println!("pre_tag_size{}: {}", index, pre_tag_size);
        match reader.read_tag_header() {
            Ok(Some(tag_header)) => {
                println!("tag{} header: {:?}", index, tag_header);
                match tag_header.tag_type {
                    TagType::Audio => {
                        let audio_header = reader.read_audio_data_header()?;
                        println!("audio header: {:?}", audio_header);
                        println!(
                            "audio data: {} bytes",
                            reader.read_data(tag_header.data_size as usize - 1)?.len()
                        );
                    }
                    TagType::Video => {
                        let video_header = reader.read_video_data_header()?;
                        println!("video header: {:?}", video_header);
                        println!(
                            "video data: {} bytes",
                            reader.read_data(tag_header.data_size as usize - 1)?.len()
                        );
                    }
                    TagType::ScriptData => {
                        let metadata = reader.read_metadata()?;
                        println!("metadata: {:?}", metadata);
                    }
                    TagType::Reserved(tt) => {
                        println!("unexpected tag type: 0x{:x?}", tt);
                    }
                }

                index += 1;
            }
            Ok(None) => {
                break;
            }
            Err(e) => {
                println!("error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
