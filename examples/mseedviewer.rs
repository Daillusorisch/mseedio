use clap::{self, Parser};
use mseedio;

#[derive(clap::Parser)]
struct Cmd {
    /// miniseed3 file to read
    file: String,
    /// Print summary of the file
    #[arg(short, long)]
    summary: bool,
    /// Print the decoded data of the file
    #[arg(short, long)]
    data: bool,
}

fn main() {
    let _ = env_logger::builder().try_init();
    let cmd = Cmd::parse();
    let bytes = std::fs::read(&cmd.file).unwrap();
    let reader = mseedio::MS3Volume::from_bytes(bytes).expect("Cannot open file");
    for record in reader {
        if cmd.summary {
            println!("{:?}", record.summary());
        }
        if cmd.data {
            println!("{:?}", record.data().expect("decode error"));
        }
    }
}
