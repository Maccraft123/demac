use std::ffi::OsStr;
use std::time::SystemTime;

use binrw::{
    BinRead, BinWrite, BinResult,
    io::{Read, Seek, SeekFrom},
    binread,
};
use bitflags::bitflags;
use bitfield_struct::bitfield;
use derivative::Derivative;

use crate::common::{DateTime, PascalString, BootBlocks, SizedString, DynamicPascalString};

pub mod fuse;

#[binread]
#[derive(Clone, Derivative)]
#[derivative(Debug)]
#[brw(big)]
pub struct Mfs {
    #[brw(pad_size_to = 0x400)]
    boot: BootBlocks,
    #[brw(pad_size_to = 0x40)]
    info: VolumeInformation,
    #[br(count = (info.alloc_block_count*12)/8, map = gimme_block_map)]
    #[derivative(Debug = "ignore")]
    block_map: BlockMap,
    #[brw(seek_before = SeekFrom::Start(512*info.file_directory_start as u64))]
    #[br(count = info.file_directory_length, map = Self::drop_nonexistent_files)]
    files: Vec<FileDirectoryBlock>,
    #[brw(seek_before = SeekFrom::Start(512 * info.alloc_block_start as u64))]
    #[br(count = info.alloc_block_count as u32 * info.alloc_block_size)]
    #[derivative(Debug = "ignore")]
    contents: Vec<u8>,
}

impl Mfs {
    pub fn new<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        Self::read(reader)
    }
    pub fn alloc_block_size(&self) -> u32 {
        self.info.alloc_block_size
    }
    pub fn creation_date(&self) -> std::time::SystemTime {
        self.info.creation_date.to_system_time()
    }
    pub fn files(&self) -> &[FileDirectoryBlock] {
        &self.files
    }
    pub fn add_file(&mut self, name: &str, ty: [u8; 4], creator: [u8; 4]) {
        self.files.push(FileDirectoryBlock {
            flags: FileFlags::EXISTS,
            version: 0,
            file_type: SizedString::new(ty),
            file_creator: SizedString::new(creator),
            finder_flags: 0,
            position: 0,
            folder_number: 0,
            file_number: self.info.next_file_num,
            data_fork_start: 0,
            data_fork_size: 0,
            data_fork_allocated_space: 0,
            resource_fork_start: 0,
            resource_fork_size: 0,
            resource_fork_allocated_space: 0,
            creation_date: DateTime::now(),
            modification_date: DateTime::now(),
            name: DynamicPascalString::new(name),
        });
        self.info.next_file_num += 1;
    }
    pub fn file_by_id(&self, num: u32) -> Option<&FileDirectoryBlock> {
        self.files
            .iter()
            .find(|f| f.file_number == num)
    }
    pub fn file_by_name(&self, name: &str) -> Option<&FileDirectoryBlock> {
        self.files
            .iter()
            .find(|f| f.name() == name)
    }
    pub fn file_data(&self, file: &FileDirectoryBlock) -> Vec<u8> {
        self.block_map.blocks_of(file.data_fork_start)
            .flat_map(|block| self.alloc_block_data(block))
            .take(file.data_fork_size as usize)
            .map(|v| *v)
            .collect()
    }
    pub fn file_rsrc(&self, file: &FileDirectoryBlock) -> Vec<u8> {
        self.block_map.blocks_of(file.resource_fork_start)
            .flat_map(|block| self.alloc_block_data(block))
            .take(file.resource_fork_size as usize)
            .map(|v| *v)
            .collect()
    }
    fn alloc_block_data(&self, block: u16) -> &[u8] {
        eprintln!("block {:x}", block);
        let start = (block as usize * self.info.alloc_block_size as usize);
        eprintln!("start {:x}", start);
        &self.contents[start as usize ..][..self.info.alloc_block_size as usize]
    }
    fn drop_nonexistent_files(files: Vec<FileDirectoryBlock>) -> Vec<FileDirectoryBlock> {
        files.into_iter()
            .filter(|f| f.flags.contains(FileFlags::EXISTS))
            .collect()
    }
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

pub struct BlockIter<'a> {
    cur_idx: u16,
    data: &'a [u16],
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        let old = self.cur_idx;
        match self.cur_idx {
            0 | 1 => None,
            _ => {
                let old = self.cur_idx;
                self.cur_idx = *self.data.get(self.cur_idx as usize - 2)?;
                println!("{:x} => {:x}", old, self.cur_idx);
                Some(old - 2)
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct BlockMap(Vec<u16>);

impl BlockMap {
    fn blocks_of(&self, start: u16) -> BlockIter {
        BlockIter {
            cur_idx: start,
            data: &self.0,
        }
    }
}

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
    data_fork_start: u16,
    data_fork_size: u32,
    data_fork_allocated_space: u32,
    resource_fork_start: u16,
    resource_fork_size: u32,
    resource_fork_allocated_space: u32,
    creation_date: DateTime,
    modification_date: DateTime,
    #[brw(align_after = 2)]
    name: DynamicPascalString,
}

impl FileDirectoryBlock {
    pub fn data_fork_size(&self) -> u32 {
        self.data_fork_size
    }
    pub fn resource_fork_size(&self) -> u32 {
        self.resource_fork_size
    }
    pub fn creation_date(&self) -> SystemTime {
        self.creation_date.to_system_time()
    }
    pub fn modification_date(&self) -> SystemTime {
        self.modification_date.to_system_time()
    }
    pub fn name(&self) -> &str {
        self.name.try_as_str().unwrap_or("")
    }
    pub fn number(&self) -> u32 {
        self.file_number
    }
}

bitflags! {
    #[derive(Clone, Debug)]
    pub struct FileFlags: u8 {
        const EXISTS = 0x80;
        const LOCKED = 0x01;
    }
}
