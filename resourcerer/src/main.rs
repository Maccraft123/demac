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
        id: u16,
        destination: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut file = File::open(args.file)?;
    let res = Resource::read(&mut file)?;
    for r in res.iter() {
        if let Some(name) = r.name() {
            println!("Resource \"{}\":", name);
        } else {
            println!("Unnamed resource:");
        }
        println!("ID {:?}", r.id());
        match r.data() {
            Type::Other(_) => println!("Data of type {:?}", r.ty()),
            known => {
                println!("{:#x?}", known);
            },
        }
    }
    /*match args.cmd {
        Command::List => {
            println!("Data:");
            for r in &res {
                println!();
                println!("Resource Type: {}", r.ty());
                for reference in r.refs() {
                    println!("\tID: 0x{:02x}, Name: {:?}, Data length: {}",
                        reference.id(),
                        reference.name(),
                        reference.data().len(),
                    );
                }
            }
        },
        Command::Extract { id, destination } => {
            let Some(reference) = res.iter()
                .flat_map(|resource| resource.refs())
                .find(|r| r.id() == id)
                else {
                    bail!("Falied to find data with supplied ID")
                };
            std::fs::write(destination, reference.data())?;
        },
    }*/

    Ok(())
}
