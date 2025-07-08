use clap::{Parser, Subcommand};
use std::path::PathBuf;
use binrw::BinRead;
use std::io::{Read, Seek};
use std::fs::{self, File};
use anyhow::{bail, Result};
use macfmt::single::{AppleFile, EntryData};

#[derive(Debug, Parser)]
struct Args {
    file: PathBuf,
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    DumpResources {
        destination: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut file = File::open(&args.file)?;
    let data = AppleFile::read(&mut file)?;

    match args.cmd {
        Command::DumpResources { destination } => {
            let res = data.entries()
                .find_map(|e| {
                    if let EntryData::ResourceFork(vec) = e {
                        Some(vec)
                    } else {
                        None
                    }
                });
            if let Some(res) = res {
                fs::write(&destination, &res)?;
            } else {
                bail!("Input file has no resource fork");
            }
        },
    }

    Ok(())
}
