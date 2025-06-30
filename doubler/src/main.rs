use clap::Parser;
use std::path::PathBuf;
use binrw::BinRead;
use std::io::{Read, Seek};
use std::fs::File;
use anyhow::{bail, Result};
use macfmt::single::{AppleDoubleHeader, AppleSingle};

#[derive(Debug, Parser)]
pub struct Args {
    file: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut file = File::open(&args.file)?;
    let mut magic = [0, 0, 0, 0];
    file.read_exact(&mut magic)?;
    file.rewind()?;
    match magic {
        [0x00, 0x05, 0x16, 0x07] => {
            let data = AppleDoubleHeader::read(&mut file)?;
            for item in data.entries() {
                println!("{:#x?}", item);
            }
        },
        [0x00, 0x05, 0x16, 0x00] => {
            let data = AppleSingle::read(&mut file)?;
            for item in data.entries() {
                println!("{:#x?}", item);
            }
        },
        _ => bail!("Unknown magic bytes {:x?}", magic),
    }

    Ok(())
}
