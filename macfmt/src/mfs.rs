use binrw::{
    BinRead, BinWrite, BinResult,
    io::{Read, Seek, SeekFrom},
    binread,
};
use bitflags::bitflags;
use bitfield_struct::bitfield;

use crate::common::{DateTime, PascalString, BootBlocks, SizedString, DynamicPascalString};

#[binread]
#[derive(Clone, Debug)]
#[brw(big)]
pub struct Mfs {
    #[brw(pad_size_to = 0x400)]
    boot: BootBlocks,
    #[brw(pad_size_to = 0x40)]
    info: VolumeInformation,
    #[br(count = (info.alloc_block_count*12)/8, map = gimme_block_map)]
    block_map: BlockMap,
    #[brw(seek_before = SeekFrom::Start(512*info.file_directory_start as u64))]
    #[br(count = info.file_directory_length, map = Mfs::drop_nonexistent_files)]
    files: Vec<FileDirectoryBlock>,
}

#[inline(always)]
fn nib_hi(v: u8) -> u8 {
    (v & 0xf0) >> 4
}

#[inline(always)]
fn nib_lo(v: u8) -> u8 {
    v & 0x0f
}

#[inline(always)]
fn u16_from_nibs(n1: u8, n2: u8, n3: u8) -> u16 {
    (n1 as u16) << 8 |
        (n2 as u16) << 4 |
        (n3 as u16)
}

fn gimme_block_map(v: Vec<u8>) -> BlockMap {
    let vec: Vec<u16> = v.chunks_exact(3)
        .flat_map(|slice| {
            [
                u16_from_nibs(nib_hi(slice[0]), nib_lo(slice[0]), nib_hi(slice[1])),
                u16_from_nibs(nib_lo(slice[1]), nib_hi(slice[2]), nib_lo(slice[2])),
            ]
        })
        .collect();
    BlockMap(vec)
}

impl Mfs {
    pub fn new<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        Self::read(reader)
    }
    fn drop_nonexistent_files(files: Vec<FileDirectoryBlock>) -> Vec<FileDirectoryBlock> {
        files.into_iter()
            .filter(|f| f.flags.contains(FileFlags::EXISTS))
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct BlockMap(Vec<u16>);

#[derive(Clone, Debug, BinRead, BinWrite)]
#[brw(big, magic = b"\xD2\xD7")]
pub struct VolumeInformation {
    creation_date: DateTime,
    last_backup_date: DateTime,
    attributes: u16,
    file_count: u16,
    file_directory_start: u16,
    file_directory_length: u16,
    alloc_block_count: u16,
    alloc_block_size: u32,
    clump_size: u32,
    alloc_block_start: u16,
    next_file_num: u32,
    free_alloc_blocks: u16,
    name: PascalString<27>,
}

#[derive(Clone, Debug, BinRead, BinWrite)]
#[brw(big)]
pub struct FileDirectoryBlock {
    #[br(map = |b: u8| FileFlags::from_bits_retain(b))]
    #[bw(map = |b: &FileFlags| b.bits())]
    flags: FileFlags,
    version: u8,
    file_type: SizedString<4>,
    file_creator: SizedString<4>,
    finder_flags: u16,
    position: u32,
    folder_number: i16,
    file_number: u32,
    first_data_fork_block: u16,
    data_fork_size: u32,
    data_fork_allocated_space: u32,
    first_resource_fork_block: u16,
    resource_fork_size: u32,
    resource_fork_allocated_space: u32,
    creation_date: DateTime,
    modification_date: DateTime,
    #[brw(align_after = 2)]
    name: DynamicPascalString,
}

bitflags! {
    #[derive(Clone, Debug)]
    pub struct FileFlags: u8 {
        const EXISTS = 0x80;
        const LOCKED = 0x01;
    }
}
