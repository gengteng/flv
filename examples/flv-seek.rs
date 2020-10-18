use flv::stdio::FlvReader;
use flv::TagType;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

/// flv seek
#[derive(Debug, StructOpt)]
#[structopt(name = "flv-seek", about = "A flv file seek tool.")]
struct Opts {
    /// flv file path
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    file: PathBuf,

    /// start timestamp in millisecond
    #[structopt(short = "s", long = "start")]
    start: u32,
}

fn main() -> anyhow::Result<()> {
    let Opts { file, start } = Opts::from_args();

    let file = File::open(file)?;
    let mut flv = FlvReader::new(file);
    let _header = flv.read_header()?;
    let _pre_tag_size = flv.read_pre_tag_size()?;
    if let Some(tag_header) = flv.read_tag_header()? {
        match tag_header.tag_type {
            TagType::Audio => {}
            TagType::Video => {}
            TagType::ScriptData => {
                let metadata = flv.read_metadata()?;
                if let Some((ts, offset)) = metadata.seek(start) {
                    flv.seek(offset)?;
                    println!(
                        "flv seek to offset {} (expected timestamp: {}, actual timestamp: {})",
                        offset, start, ts
                    );
                }
            }
            TagType::Reserved(_) => {}
        }
    }

    Ok(())
}
