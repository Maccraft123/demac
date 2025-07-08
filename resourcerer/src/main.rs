use std::path::PathBuf;
use std::fs::File;

use anyhow::{bail, Result};
use binrw::BinRead;
use clap::{Parser, Subcommand};
use macfmt::rsrc::{types::Type, Resource};

#[derive(Parser, Debug)]
struct Args {
    file: PathBuf,
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    List,
    Extract {
        id: i16,
        destination: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut file = File::open(args.file)?;
    let res = Resource::read(&mut file)?;
    match args.cmd {
        Command::List => {
            println!("Data:");
            for r in &res {
                if let Some(name) = r.name() {
                    println!("Resource \"{}\":", name);
                } else {
                    println!("Unnamed resource:");
                }
                println!("ID {:?}", r.id());
            }
        },
        Command::Extract { id, destination } => {
            let Some(reference) = res.iter()
                .find(|r| r.id() == id)
                else {
                    bail!("Falied to find data with supplied ID")
                };
            match reference.data() {
                Type::ColorLut(lut) => {
                    for e in lut.entries() {
                        println!(
                            "0x{:02x} => image::Rgb([0x{:02x}, 0x{:02x}, 0x{:02x}])",
                            e.pixel(), (e.r() >> 8) as u8, (e.g() >> 8) as u8, (e.b() >> 8) as u8,
                        );
                    }
                },
                _ => todo!(),
            }
        },
    }

    Ok(())
}
