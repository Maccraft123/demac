use std::io;
use std::time::SystemTime;

use binrw::{
    BinRead, BinResult, BinWrite, binread,
    io::{Read, Seek, SeekFrom},
};
use bitflags::bitflags;
use derivative::Derivative;

use crate::common::{DateTime, DynamicPascalString, PascalString, SizedString};
use super::BootBlocks;

//pub mod fuse;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Fork {
    Resource,
    Data,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FileHandle(usize);

#[derive(Debug)]
pub struct FileWriter<'a> {
    mfs: &'a mut Mfs,
    file: FileHandle,
    offset: u64,
    fork: Fork,
}

impl<'a> io::Write for FileWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let _offset_blocks = self.offset / 512;
        let _offset_rem = self.offset % 512;
        let mut data = io::Cursor::new(self.mfs.file_contents(self.file, self.fork));
        data.seek(SeekFrom::Start(self.offset))?;
        data.write_all(buf)?;
        self.mfs
            .overwrite_contents(self.file, self.fork, data.into_inner())?;
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> io::Seek for FileWriter<'a> {
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        todo!()
    }
}

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
    pub fn add_file(&mut self, name: &str, ty: [u8; 4], creator: [u8; 4]) -> FileHandle {
        let file_number = self.info.next_file_num;

        self.files.push(FileDirectoryBlock {
            flags: FileFlags::EXISTS,
            version: 0,
            file_type: SizedString::new(ty),
            file_creator: SizedString::new(creator),
            finder_flags: 0,
            position: 0,
            folder_number: 0,
            file_number,
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
        self.info.file_count += 1;
        self.info.next_file_num += 1;

        FileHandle(self.files.len() - 1)
    }
    pub fn file_by_id(&self, num: u32) -> Option<FileHandle> {
        self.files
            .iter()
            .enumerate()
            .find_map(|(id, f)| (f.file_number == num).then_some(FileHandle(id)))
    }
    pub fn file_by_name(&self, name: &str) -> Option<FileHandle> {
        self.files
            .iter()
            .enumerate()
            .find_map(|(id, f)| (f.name() == name).then_some(FileHandle(id)))
    }
    pub fn file_writer<'a>(&'a mut self, file: FileHandle, fork: Fork) -> FileWriter<'a> {
        FileWriter {
            mfs: self,
            file,
            offset: 0,
            fork,
        }
    }
    pub fn append_file_data(&mut self, file: &FileDirectoryBlock, mut data: &[u8]) {
        let free_size = file.data_fork_allocated_space - file.data_fork_size;
        if data.len() <= free_size as usize {
            let old_file_end_block = (file.data_fork_size / self.info.alloc_block_size) as usize;
            let old_file_end_rem = (file.data_fork_size % self.info.alloc_block_size) as usize;
            let block_size = self.info.alloc_block_size as usize;
            let blocks: Vec<u16> = self
                .block_map
                .blocks_of(file.data_fork_start)
                .skip(old_file_end_block)
                .collect();
            let mut iter = blocks.into_iter();
            let to_write = (block_size as usize).min(data.len());

            self.alloc_block_data_mut(iter.next().unwrap())[old_file_end_rem..][..to_write]
                .copy_from_slice(&data[..to_write]);

            if data.len() <= to_write {
                return;
            }

            data = &data[old_file_end_rem..];

            for block in iter {
                let to_write = (block_size as usize).min(data.len());
                self.alloc_block_data_mut(block)[..to_write].copy_from_slice(&data[..to_write]);
                if data.len() <= to_write {
                    return;
                }
                data = &data[block_size..];
            }
        }
    }
    pub fn file_contents(&self, file: FileHandle, fork: Fork) -> Vec<u8> {
        let file = &self.files[file.0];
        self.block_map
            .blocks_of(file.fork_start(fork))
            .flat_map(|block| self.alloc_block_data(block))
            .take(file.fork_size(fork) as usize)
            .map(|v| *v)
            .collect()
    }
    fn overwrite_contents(
        &mut self,
        file: FileHandle,
        fork: Fork,
        data: Vec<u8>,
    ) -> io::Result<()> {
        let file = &self.files[file.0];
        if self.block_map.blocks_of(file.fork_start(fork)).count() < (data.len() + 511) / 512 {
            self.block_map.ensure_len(file.fork_start(fork))?;
        }
        Ok(())
    }
    pub fn file_data(&self, file: FileHandle) -> Vec<u8> {
        self.file_contents(file, Fork::Data)
    }
    pub fn file_rsrc(&self, file: FileHandle) -> Vec<u8> {
        self.file_contents(file, Fork::Resource)
    }
    fn alloc_block_data_mut(&mut self, block: u16) -> &mut [u8] {
        let start = block as usize * self.info.alloc_block_size as usize;
        &mut self.contents[start as usize..][..self.info.alloc_block_size as usize]
    }
    fn alloc_block_data(&self, block: u16) -> &[u8] {
        let start = block as usize * self.info.alloc_block_size as usize;
        &self.contents[start as usize..][..self.info.alloc_block_size as usize]
    }
    fn drop_nonexistent_files(files: Vec<FileDirectoryBlock>) -> Vec<FileDirectoryBlock> {
        files
            .into_iter()
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
    (n1 as u16) << 8 | (n2 as u16) << 4 | (n3 as u16)
}

fn gimme_block_map(v: Vec<u8>) -> BlockMap {
    let vec: Vec<u16> = v
        .chunks_exact(3)
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
        match self.cur_idx {
            0 | 1 => None,
            _ => {
                let old = self.cur_idx;
                self.cur_idx = *self.data.get(self.cur_idx as usize - 2)?;
                println!("{:x} => {:x}", old, self.cur_idx);
                Some(old - 2)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct BlockMap(Vec<u16>);

impl BlockMap {
    fn ensure_len(&mut self, _block: u16) -> io::Result<()> {
        todo!()
    }
    /*fn allocate_to(&mut self, num: Option<u16>) -> u16 {
        let free = self
            .0
            .iter()
            .enumerate()
            .find(|(_, v)| **v == 0)
            .map(|(i, _)| i)
            .unwrap() as u16;
        if let Some(n) = num {
            self.0[n as usize] = free;
        }
        free
    }*/
    fn blocks_of<'a>(&'a self, start: u16) -> BlockIter<'a> {
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
    //fn fork_free_space(&self, fork: Fork) -> u32 {
    //    self.fork_allocated_space(fork) - self.fork_size(fork)
    //}
    pub fn fork_start(&self, fork: Fork) -> u16 {
        match fork {
            Fork::Data => self.data_fork_start,
            Fork::Resource => self.resource_fork_start,
        }
    }
    pub fn fork_size(&self, fork: Fork) -> u32 {
        match fork {
            Fork::Data => self.data_fork_size,
            Fork::Resource => self.resource_fork_size,
        }
    }
    pub fn fork_allocated_space(&self, fork: Fork) -> u32 {
        match fork {
            Fork::Data => self.data_fork_allocated_space,
            Fork::Resource => self.resource_fork_allocated_space,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::{Fork, Mfs};
    use std::io::{Cursor, Write};
    const INFINITE_DSK: &'static [u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/infinite.dsk"
    ));
    const READ_ME: &'static [u8] = b"This disk contains some software for early Macs. It uses the MFS format (the file system supported by System 1.0 through 2.0).\r\rFor a more complete set of software, use System 2.1 or higher. It supports HFS, and has access to a much larger (1GB+) library.";
    #[test]
    fn read() {
        let mut disk = Cursor::new(INFINITE_DSK.to_vec());
        let mfs = Mfs::new(&mut disk).unwrap();
        let file = mfs.file_by_name("Read Me").unwrap();
        let data = mfs.file_data(file);
        assert_eq!(data, READ_ME);
    }
    #[test]
    fn write_then_read() {
        let mut disk = Cursor::new(INFINITE_DSK.to_vec());
        let mut mfs = Mfs::new(&mut disk).unwrap();
        mfs.add_file("testfile", *b"TEST", *b"TEST");
        let file = mfs.file_by_name("testfile").unwrap();
        mfs.file_writer(file, Fork::Data)
            .write_all(b"test data")
            .unwrap();
        assert_eq!(mfs.file_data(file), b"test data");
    }
}
