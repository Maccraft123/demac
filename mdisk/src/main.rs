use std::path::PathBuf;
use std::io::{Read, Seek, SeekFrom};
use std::fs::File;

use anyhow::{bail, Result};
use binrw::BinRead;
use clap::{Parser, ValueEnum};
use comfy_table::Table;
use macfmt::apm::{ApmDrive, Driver, Partition};
use macfmt::mfs::Mfs;

#[derive(Debug, Clone, Parser)]
struct Args {
    #[arg(short, long, default_value = "autodetect")]
    format: Format,
    input: PathBuf,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Format {
    Hfs,
    Mfs,
    Apm,
    Autodetect,
}

fn show_partitions(partitions: &[Partition]) {
    let mut table = Table::new();
    table.set_header(vec!["ID", "Type", "Name", "Start", "End"]);
    for (i, p) in partitions.iter().enumerate() {
        table.add_row(vec![
            format!("{}", i),
            format!("{}", p.kind()),
            format!("{}", p.name().unwrap()),
            format!("{:x}", p.start()),
            format!("{:x}", p.start() + p.size() - 1),
        ]);
    }

    println!("{}", table);
}

fn show_drivers(drivers: &[Driver]) {
    let mut table = Table::new();
    table.set_header(vec!["ID", "OS Type", "Start", "End"]);
    for (i, d) in drivers.iter().enumerate() {
        table.add_row(vec![
            format!("{}", i),
            format!("{}", d.os_type()),
            format!("{:x}", d.start()),
            format!("{:x}", d.start() + d.size() as u32 - 1),
        ]);
    }

    println!("{}", table);
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut file = File::open(args.input)?;
    let fmt = match args.format {
        Format::Autodetect => {
            let mut magic = [0u8; 2];
            file.read_exact(&mut magic)?;
            file.rewind()?;
            match &magic {
                b"LK" | b"\0\0" => {
                    let mut more_magic = [0u8; 2];
                    file.seek(SeekFrom::Start(1024))?;
                    file.read_exact(&mut more_magic)?;
                    file.rewind()?;

                    match &more_magic {
                        b"\xD2\xD7" => Format::Mfs,
                        b"BD" => Format::Hfs,
                        _ => bail!("Unknown sector 1 magic bytes {:02x?}", magic),
                    }
                },
                b"ER" => Format::Apm,
                _ => bail!("Unknown sector 0 magic bytes {:02x?}", magic),
            }
        },
        _ => args.format,
    };

    match fmt {
        Format::Apm => {
            let disk = ApmDrive::new(&mut file)?;

            show_partitions(disk.partitions());
            show_drivers(disk.drivers());
        },
        Format::Mfs => {
            let vol = Mfs::new(&mut file)?;
            println!("{:#x?}", vol);
        },
        Format::Autodetect => unreachable!(),
        _ => todo!("{:?} disk format", fmt),
    }

    Ok(())
}
