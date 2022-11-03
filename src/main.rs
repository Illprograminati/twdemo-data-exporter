use std::{
    fs::File,
    io::{self, Write},
    path::PathBuf,
};

use ahash::RandomState;
use clap::Parser;
use demo::Reader;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    /// Pretty print the json
    pretty: bool,
    #[clap(short, long)]
    /// Write output to this file instead of printing to stdout
    out: Option<PathBuf>,
    /// The demo file to read
    demo: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut warn = warn::Log;
    let mut reader = Reader::open(&mut warn, &args.demo).unwrap();

    let hash_builder = RandomState::with_seed(3124576891576154); // seed randomly chosen by <Scrax>
    let mut hashes = vec![];
    while let Ok(Some(chunk)) = reader.read_chunk(&mut warn) {
        match chunk {
            demo::Chunk::Tick(keyframe, tick) => {
                hashes.push(hash_builder.hash_one((keyframe, tick)));
            }
            demo::Chunk::Snapshot(b) => {
                hashes.push(hash_builder.hash_one(b));
            }
            demo::Chunk::SnapshotDelta(b) => {
                hashes.push(hash_builder.hash_one(b));
            }
            demo::Chunk::Message(m) => {
                hashes.push(hash_builder.hash_one(m));
            }
        }
    }
    let json = if args.pretty {
        serde_json::to_string_pretty(&hashes)
    } else {
        serde_json::to_string(&hashes)
    }?;
    let mut w = args
        .out
        .map(|path| File::create(&path).map(|file| Box::new(file) as Box<dyn Write>))
        .unwrap_or_else(|| Ok(Box::new(io::stdout()) as Box<dyn Write>))?;
    writeln!(w, "{}", json)?;
    Ok(())
}
