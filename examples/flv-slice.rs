use std::path::PathBuf;
use structopt::StructOpt;

/// flv slice
#[derive(Debug, StructOpt)]
#[structopt(name = "flv-slice", about = "A flv file slice tool.")]
struct Opts {
    /// flv file path
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    file: PathBuf,

    /// start timestamp in millisecond
    #[structopt(short = "s", long = "start")]
    start: u32,

    /// end timestamp in millisecond
    #[structopt(short = "e", long = "end")]
    end: u32,

    /// output flv file path
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::from_args();
    println!("{:?}", opts);
    Ok(())
}
