use std::path::PathBuf;
use std::io::{Read, Seek, SeekFrom};
use std::fs::File;

use anyhow::{bail, Result};
use binrw::BinRead;
use clap::{Parser, ValueEnum, Subcommand};
use comfy_table::Table;
use macfmt::apm::{ApmDrive, Driver, Partition};
use macfmt::fs::hfs::HfsVolume;

#[derive(Debug, Clone, Parser)]
struct Args {
    #[arg(short, long, default_value = "autodetect")]
    format: Format,
    input: PathBuf,
    #[arg(short, long)]
    mount: Option<PathBuf>,
    #[command(subcommand)]
    op: Operation,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Format {
    Hfs,
    Mfs,
    Apm,
    Autodetect,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Fork {
    Resource,
    Data,
}

#[derive(Debug, Clone, Subcommand)]
enum Operation {
    Ls {
        #[arg(default_value = "/")]
        path: String,
    },
    Get {
        src: String,
        fork: Fork,
        dst: PathBuf,
    },
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
            let mut sector0 = [0u8; 0x200];
            file.read_exact(&mut sector0)?;
            file.rewind()?;
            let apm = &sector0[0..2] == b"ER";

            if !apm {
                let mut more_magic = [0u8; 2];
                file.seek(SeekFrom::Start(1024))?;
                file.read_exact(&mut more_magic)?;
                file.rewind()?;

                match &more_magic {
                    b"\xD2\xD7" => Format::Mfs,
                    b"BD" => Format::Hfs,
                    _ => bail!("Unknown sector 1 magic bytes {:02x?}", more_magic),
                }
            } else {
                Format::Apm
            }
        },
        _ => args.format,
    };

    match fmt {
        Format::Hfs => {
            let mut fs = HfsVolume::new(file)?;
            let root = fs.root_dir();
            match args.op {
                Operation::Ls { path } => {
                    let mut dir = &root;
                    for seg in path.split("/").filter(|s| !s.is_empty()) {
                        if let Some(d) = dir.subdir(seg) {
                            dir = d;
                        } else {
                            if let Some(file) = dir.file(&seg) {
                                bail!("Not a directory: {:?}", seg);
                            } else {
                                bail!("No such directory: {:?}", seg);
                            }
                        }
                    }
                    for file in dir.files() {
                        println!("File '{}'", file.name());
                    }
                    for subdir in dir.subdirs() {
                        println!("Dir '{}'", subdir.name());
                    }
                    if dir.files().is_empty() && dir.subdirs().is_empty() {
                        println!("<empty>");
                    }
                },
                Operation::Get { src, fork, dst } => {
                    let mut dir = &root;
                    let (path, filename) = src.rsplit_once("/")
                        .expect("i don't know how to name this error message");

                    for seg in path.split("/").filter(|s| !s.is_empty()) {
                        if let Some(d) = dir.subdir(seg) {
                            dir = d;
                        } else {
                            if let Some(file) = dir.file(&seg) {
                                bail!("Not a directory: {:?}", seg);
                            } else {
                                bail!("No such directory: {:?}", seg);
                            }
                        }
                    }
                    
                    let Some(file) = dir.file(filename) else {
                        bail!("No such file: '{:?}'", filename);
                    };

                    let data = fs.file_data(&file).unwrap();
                    if data.len() == 0 {
                        bail!("Refusing to write an empty file");
                    }
                    std::fs::write(dst, &data)?;
                },
            }
        },
        Format::Autodetect => unreachable!(),
        _ => todo!("{:?} disk format", fmt),
    }

    Ok(())
}
