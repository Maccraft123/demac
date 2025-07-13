pub mod mfs;
pub mod hfs;

use binrw::{BinRead, BinWrite};
use derivative::Derivative;
use std::fmt;
use crate::common::PascalString;

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big, magic = b"LK")]
pub struct BootBlocks {
    entry_point: u32,
    version: u16,
    page_flags: u16,
    system_filename: PascalString<15>,
    shell_filename: PascalString<15>,
    debugger_filename: PascalString<15>,
    debugger_filename2: PascalString<15>,
    startup_screen: PascalString<15>,
    startup_program_filename: PascalString<15>,
    system_scrap_filename: PascalString<15>,
    fcb_count: u16,
    event_queue_count: u16,
    system_heap_size_128k: u32,
    system_heap_size_256k: u32,
    system_heap_size: u32,
    #[br(if(version & 0x2000 != 0))]
    extra_data: Option<BootBlockExtra>,
    #[derivative(Debug(format_with = "BootBlocks::code_vec_fmt"))]
    #[br(count = 2)]
    code: Vec<u16>,
}

impl BootBlocks {
    fn code_vec_fmt(_v: &Vec<u16>, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "(...)")
    }
}

#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(big)]
pub struct BootBlockExtra {
    #[brw(pad_before = 2)]
    system_heap_extra: u32,
    system_heap_fract: u32,
}

