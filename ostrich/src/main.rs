use std::path::PathBuf;

use clap::Parser;
use anyhow::Result;
use m68000::{M68000, cpu_details::Mc68000};

#[derive(Parser)]
struct Args {
    executable: PathBuf,
}

fn main() -> Result<()> {
    let cpu: M68000<Mc68000> = M68000::new_no_reset();
    println!("Hello, world!");
    Ok(())
}
