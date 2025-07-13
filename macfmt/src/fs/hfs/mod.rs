use std::fmt;

use crate::common::PascalString;
use binrw::{BinRead, BinWrite};
use derivative::Derivative;

mod btree;

//fn div_ceil(x: u16, y: u32) -> u32 {
//    (x as u32 + y - 1) / y
//}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct HfsVolume {
    boot_blks: BootBlockHeader,
    //#[brw(align_before = 512)]
    //mdb: HfsMasterDirectoryBlock,
    //#[br(count = div_ceil(mdb.alloc_block_num, 512*8))]
    //#[brw(align_before = 512)]
    //vol_bmp: Vec<BitmapBlock>,
    //#[brw(align_before = 512)]
    //catalog_file: btree::HeaderNode,
    //#[brw(align_before = 512)]
    //extents_overflow_file: btree::HeaderNode,
}

/*#[derive(Clone, BinRead, BinWrite)]
struct BitmapBlock([u8; 512]);
impl fmt::Debug for BitmapBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "BitmapBlock(..)")
    }
}

#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(big, magic = b"BD")]
pub struct HfsMasterDirectoryBlock {
    creation_time: DateTime,
    modification_time: DateTime,
    attributes: u16,
    root_dir_file_count: u16,
    volume_bitmap_start: u16,
    next_alloc_search: u16,
    alloc_block_num: u16,
    alloc_block_size: u32,
    clump_size: u32,
    first_alloc_block: u16,
    next_unused_catalog_node_id: u32,
    unused_alloc_blocks: u16,
    name: PascalString<27>,
    last_backup_time: DateTime,
    backup_seq: u16,
    write_count: u32,
    extents_overflow_clump_size: u32,
    catalog_file_clump_size: u32,
    dirs_in_root_dir_count: u16,
    file_count: u32,
    dir_count: u32,
    finder_info: [u32; 8],
    vol_cache_size: u16,
    vol_bitmap_cache_size: u16,
    common_vol_cache_size: u16,
    extents_overflow_file_size: u32,
    extents_overflow_record: [ExtDescriptor; 3],
    catalog_file_size: u32,
    catalog_file_record: [ExtDescriptor; 3],
}

#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(big)]
pub struct ExtDescriptor {
/*    key_len: i8,
    fork_type: i8,
    file_num: u32,
    starting_file_allocation_block: u16,*/
    block: u16,
    count: u16,
}*/

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big, magic = b"LK")]
pub struct BootBlockHeader {
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
    #[derivative(Debug(format_with = "BootBlockHeader::code_vec_fmt"))]
    #[br(count = 400)]
    code: Vec<u16>,
}

impl BootBlockHeader {
    pub fn code_iter(&self) -> impl Iterator<Item = un68k::Instruction> {
        un68k::decode_iter(self.code.clone())
    }
    fn code_vec_fmt(_v: &Vec<u16>, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "(...)")
    }
}

#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(big)]
pub struct BootBlockExtra {
    pad: u16,
    system_heap_extra: u32,
    system_heap_fract: u32,
}

/*#[cfg(test)]
mod tests {
    use super::{HfsVolume, BootBlockHeader};

    macro_rules! testdata {
        ($filename: literal) => {
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/", $filename))
        }
    }

    static MACOS7_DISK_TOOLS: &'static [u8] = testdata!("macos_7_disk_tools.img");
    #[test]
    fn decode_boot_floppy() {
        let (_, decoded) = HfsVolume::from_bytes((&MACOS7_DISK_TOOLS, 0))
            .unwrap();

        //let code: Vec<un68k::Instruction> = decoded.boot_blks.code_iter().collect();
        //panic!("{:#?}", code);
        panic!("{:#x?}", decoded);
    }
}*/
