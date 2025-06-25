use std::path::PathBuf;
use std::fs::File;

use anyhow::Result;
use binrw::BinRead;
use clap::Parser;
use comfy_table::Table;
use macfmt::apm::{ApmDrive, Driver, Partition};

#[derive(Debug, Parser)]
struct Args {
    input: PathBuf,
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
    let disk = ApmDrive::new(&mut file)?;

    show_partitions(disk.partitions());
    show_drivers(disk.drivers());

    Ok(())
}
