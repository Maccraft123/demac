use std::fmt;

use crate::common::{DateTime, PascalString, DynamicPascalString, FinderInfo, ExtraFinderInfo, SizedString};
use bitflags::bitflags;
use binrw::{BinRead, BinWrite, BinResult};
use binrw::io::{Read, Seek, SeekFrom};
use derivative::Derivative;
use either::Either;
use super::BootBlocks;

#[derive(Debug, Clone)]
pub struct File {
    name: String,
    id: Cnid,
    data_len: u32,
    rsrc_len: u32,
}

impl File {
    fn new(name: &DynamicPascalString, id: Cnid, data_len: u32, rsrc_len: u32) -> File {
        File {
            name: name.try_as_str().unwrap().trim().to_string(),
            id,
            data_len,
            rsrc_len,
        }
    }
    pub fn rsrc_len(&self) -> u32 {
        self.rsrc_len
    }
    pub fn data_len(&self) -> u32 {
        self.data_len
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct Directory {
    name: String,
    id: Cnid,
    files: Vec<File>,
    subdirs: Vec<Directory>,
}

impl Directory {
    fn new(name: &DynamicPascalString, id: Cnid) -> Directory {
        Directory {
            name: name.try_as_str().unwrap().trim().to_string(),
            id,
            files: Vec::new(),
            subdirs: Vec::new(),
        }
    }
    fn subdirs_mut(&mut self) -> &mut Vec<Directory> {
        &mut self.subdirs
    }
    fn visit_subdirs(&mut self, f: &mut impl FnMut(&mut Directory)) {
        f(self);
        for subdir in self.subdirs.iter_mut() {
            subdir.visit_subdirs(f);
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn subdirs(&self) -> &[Directory] {
        &self.subdirs
    }
    pub fn files(&self) -> &[File] {
        &self.files
    }
    pub fn subdir(&self, name: &str) -> Option<&Directory> {
        self.subdirs.iter().find(|dir| dir.name == name)
    }
    pub fn file(&self, name: &str) -> Option<&File> {
        self.files.iter().find(|file| file.name == name)
    }
}

pub struct FileReader<'a, R: Read + Seek> {
    vol: &'a mut HfsVolume<R>,
    cur_offset: usize,
    len: usize,
    extents: ExtDataRec,
}

impl<'a, R: Read + Seek> Read for FileReader<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.cur_offset >= self.len {
            return Ok(0);
        }

        if self.extents.0[1].first_alloc_blk != 0 {
            todo!("files with more than one extent")
        }
        let start = self.vol.alloc_blk_offset(self.extents.0[0].first_alloc_blk) + self.cur_offset as u64;
        let read_len = if (self.len - self.cur_offset) > buf.len() {
            buf.len()
        } else {
            self.len - self.cur_offset
        };
        self.vol.reader.seek(SeekFrom::Start(start))?;
        self.vol.reader.read_exact(&mut buf[..read_len])?;
        self.cur_offset += read_len;

        Ok(read_len)
    }
}

impl<'a, R: Read + Seek> Seek for FileReader<'a, R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Start(off) => self.cur_offset = off as usize,
            SeekFrom::End(off) => self.cur_offset = (self.len as i64 + off) as usize,
            SeekFrom::Current(off) => self.cur_offset = (self.cur_offset as i64 + off) as usize,
        }

        Ok(self.cur_offset as u64)
    }
}

pub struct HfsVolume<R: Read + Seek> {
    hdr: Hfs,
    base: u64,
    root_dir: Directory,
    reader: R,
}

impl<R: Read + Seek> HfsVolume<R> {
    pub fn new(mut reader: R) -> BinResult<Self> {
        let base = reader.stream_position()?;
        let hdr = Hfs::read(&mut reader)?;
        let root_dir = hdr.root_dir();
        Ok(Self {
            hdr,
            base,
            root_dir,
            reader,
        })
    }
    fn alloc_blk_offset(&self, blk: u16) -> u64 {
        self.hdr.mdb.alloc_block_offset(blk) as u64 + self.base
    }
    pub fn file_reader<'a>(&'a mut self, file: &File) -> FileReader<'a, R> {
        let data = self.hdr.catalog_file.record_by_id(file.id).unwrap();
        let CatalogRecordData::File { data_extent, data_len, .. } = data else {
            panic!("should i even handle this case?")
        };
        let data_extent = data_extent.clone();
        let data_len = *data_len as usize;

        FileReader {
            vol: self,
            cur_offset: 0,
            len: data_len,
            extents: data_extent,
        }
    }
    pub fn file_data(&mut self, file: &File) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.file_reader(file).read_to_end(&mut buf)?;
        Ok(buf)
    }
    pub fn root_dir(&self) -> Directory {
        self.root_dir.clone()
    }
    pub fn hdr(&self) -> &Hfs {
        &self.hdr
    }
}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct Hfs {
    #[br(try)]
    #[brw(pad_size_to = 1024)]
    boot_blks: Option<BootBlocks>,
    #[brw(pad_size_to = 512)]
    mdb: Mdb,
    #[br(count = mdb.volume_bitmap_len())]
    #[brw(align_after = 512)]
    #[derivative(Debug = "ignore")]
    volume_bitmap: Vec<u8>,
    #[brw(seek_before = SeekFrom::Start(mdb.catalog_file_start() as u64))]
    catalog_file: CatalogFile,
    //#[brw(seek_before = SeekFrom::Start(mdb.extents_overflow_file_start() as u64))]
    //extents_overflow: ExtentsOverflowFile,
}

impl Hfs {
    fn alloc_blk_occupied(&self, blk: u16) -> bool {
        let byte = (blk / 8) as usize;
        let bit = blk % 8;
        self.volume_bitmap[byte] & (1 << bit) != 0
    }
    fn root_dir(&self) -> Directory {
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        for node in self.catalog_file.nodes.iter() {
            for record in node.recs.iter() {
                match record {
                    CatalogRecord::Leaf { name, data, parent_id, .. } => {
                        match data {
                            CatalogRecordData::Directory { id, flags, .. } => {
                                dirs.push((parent_id, Directory::new(name, *id)));
                            },
                            CatalogRecordData::File { id, data_len, rsrc_len, .. } => {
                                files.push((parent_id, File::new(name, *id, *data_len, *rsrc_len)));
                            },
                            // what do i do with file and dir threads?
                            _ => (),
                        }
                    },
                    // what do i do with index records?
                    _ => (),
                }
            }
        }

        for (_, dir) in dirs.iter_mut() {
            let mut iter = files
                .extract_if(.., |(parent_id, _)| **parent_id == dir.id)
                .map(|(_, file)| file);
            dir.files.extend(iter);
        }

        let mut root = dirs.extract_if(.., |(parent_id, _)| **parent_id == Cnid::ParentOfRoot)
            .map(|(_, dir)| dir)
            .next()
            .unwrap();

        loop {
            let mut starting_len = dirs.len();
            root.visit_subdirs(&mut |dir: &mut Directory| {
                let mut iter = dirs.extract_if(.., |(parent_id, _)| ** parent_id == dir.id)
                    .map(|(_, dir)| dir);
                dir.subdirs.extend(iter);
            });

            if dirs.len() > starting_len || dirs.is_empty() {
                break;
            }
        }

        if !dirs.is_empty() {
            panic!("Leftover directories detected: {:#x?}", dirs);
        }

        if !files.is_empty() {
            panic!("Leftover files detected: {:#x?}", files);
        }

        root
    }
}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big, magic = b"BD")]
pub struct Mdb {
    ctime: DateTime,
    mtime: DateTime,
    #[br(map = |v: u16| MdbAttrs::from_bits_retain(v))]
    #[bw(map = |v: &MdbAttrs| v.bits())]
    attrs: MdbAttrs,
    root_dir_file_count: u16,
    bitmap_start: u16,
    alloc_ptr: u16,
    alloc_blk_count: u16,
    alloc_blk_size: u32,
    clump_size: u32,
    alloc_blk_start: u16,
    next_catalog_id: u32,
    free_blks: u16,
    name: PascalString<27>,
    backup_date: DateTime,
    backup_seq: u16,
    write_count: u32,
    extents_overflow_clump_size: u32,
    catalog_clump_size: u32,
    root_dir_dir_count: u16,
    file_count: u32,
    dir_count: u32,
    #[derivative(Debug = "ignore")]
    finder_info: [u32; 8],
    cache_size: u16,
    bitmap_cache_size: u16,
    common_volume_cache_size: u16,
    extents_overflow_size: u32,
    extents_overflow_record: ExtDataRec,
    catalog_file_size: u32,
    catalog_file_extent: ExtDataRec,
}

impl Mdb {
    fn volume_bitmap_len(&self) -> usize {
        (self.alloc_blk_count as usize + 7) / 8
    }
    fn alloc_block_offset(&self, blk: u16) -> usize {
        (self.alloc_blk_start as usize * 512 ) + (blk as usize * self.alloc_blk_size as usize)
    }
    fn catalog_file_start(&self) -> usize {
        self.alloc_block_offset(self.catalog_file_extent.0[0].first_alloc_blk)
    }
}


#[derive(Derivative, Clone, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct ExtDataRec([ExtDescriptor; 3]);

#[derive(Derivative, Clone, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct ExtDescriptor {
    first_alloc_blk: u16,
    alloc_blk_count: u16,
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct MdbAttrs: u16 {
        const HW_LOCKED = 1 << 7;
        const UNMOUNTED_SUCCESSFULLY = 1 << 8;
        const BADBLOCKS_SPARED = 1 << 9;
        const SW_LOCKED = 1 << 15;
    }
}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct CatalogFile {
    header: HeaderNode,
    #[br(count = header.header_record.node_count - 1)]
    nodes: Vec<CatalogNode>,
}

impl CatalogFile {
    pub fn root_node(&self) -> &CatalogNode {
        &self.nodes[self.header.header_record.root as usize]
    }
    fn record_by_id(&self, requested_id: Cnid) -> Option<&CatalogRecordData> {
        for node in self.nodes.iter().filter(|node| node.desc.ty == NodeType::Leaf) {
            for record in node.recs.iter() {
                match record {
                    CatalogRecord::Leaf { data, .. } => {
                        match data {
                            CatalogRecordData::Directory { id, .. } => {
                                if *id == requested_id {
                                    return Some(data)
                                }
                            },
                            CatalogRecordData::File { id, .. } => {
                                if *id == requested_id {
                                    return Some(data)
                                }
                            },
                            _ => (),
                        }
                    },
                    _ => unreachable!(),
                }
            }
        }

        None
    }
}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct CatalogNode {
    desc: NodeDescriptor,
    #[brw(restore_position)]
    #[br(args { count: desc.record_count as usize, inner: (desc.ty,) })]
    recs: Vec<CatalogRecord>,
    #[brw(seek_before = SeekFrom::Current((0x200 - 0xe - (desc.record_count * 2)) as i64))]
    #[br(count = desc.record_count)]
    recs_offsets: Vec<u16>,
}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
#[br(import(ty: NodeType))]
pub enum CatalogRecord {
    #[br(pre_assert(ty == NodeType::Index))]
    Index {
        #[brw(pad_after = 1)]
        key_len: u8,
        parent_id: Cnid,
        name: PascalString<31>,
        id: Cnid,
    },
    #[br(pre_assert(ty == NodeType::Leaf))]
    Leaf {
        #[brw(pad_after = 1)]
        key_len: u8,
        parent_id: Cnid,
        #[brw(align_after = 2)]
        name: DynamicPascalString,
        data: CatalogRecordData,
    },
}

#[derive(Derivative, Clone, Eq, PartialEq, Copy, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub enum Cnid {
    #[brw(magic = b"\x00\x00\x00\x01")]
    ParentOfRoot,
    #[brw(magic = b"\x00\x00\x00\x02")]
    RootDir,
    #[brw(magic = b"\x00\x00\x00\x03")]
    ExtentsFile,
    #[brw(magic = b"\x00\x00\x00\x04")]
    CatalogFile,
    #[brw(magic = b"\x00\x00\x00\x05")]
    BadBlocksFile,
    Other(u32),
}

#[derive(Derivative, Clone, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub enum CatalogRecordData {
    #[brw(magic = b"\x01")]
    Directory {
        #[brw(pad_before = 1)]
        flags: u16,
        valence: u16,
        id: Cnid,
        ctime: DateTime,
        mtime: DateTime,
        backup_time: DateTime,
        finder_info: FinderInfo,
        #[brw(pad_after = 16)]
        more_finder_info: ExtraFinderInfo,
    },
    #[brw(magic = b"\x02")]
    File {
        #[brw(pad_before = 1)]
        #[br(map = |v: u8| FileFlags::from_bits_retain(v))]
        #[bw(map = |v: &FileFlags| v.bits())]
        flags: FileFlags,
        kind: u8,
        finder_info: FinderInfo,
        id: Cnid,
        data_start: u16,
        data_len: u32,
        data_allocated_len: u32,
        rsrc_start: u16,
        rsrc_len: u32,
        rsrc_allocated_len: u32,
        ctime: DateTime,
        mtime: DateTime,
        backup_time: DateTime,
        more_finder_info: ExtraFinderInfo,
        clump_size: u16,
        data_extent: ExtDataRec,
        #[brw(pad_after = 4)]
        rsrc_extent: ExtDataRec,
    },
    #[brw(magic = b"\x03")]
    DirectoryThread {
        #[brw(pad_before = 1 + 8)]
        parent_id: Cnid,
        name: PascalString<31>,
    },
    #[brw(magic = b"\x04")]
    FileThread {
        #[brw(pad_before = 1 + 8)]
        parent_id: Cnid,
        name: PascalString<31>,
    },
}

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct FileFlags: u8 {
        const LOCKED = 1 << 0;
        const THREAD_EXISTS = 1 << 1;
        const FILE_USED = 1 << 7;
    }
}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct HeaderNode {
    desc: NodeDescriptor,
    header_record: HeaderRecord,
    #[derivative(Debug = "ignore")]
    reserved: [u8; 128],
    map_record: MapRecord,
    free_offset: u16,
    map_offset: u16,
    reserved_offset: u16,
    header_offset: u16,
}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct Node {
    #[brw(pad_size_to = 512)]
    desc: NodeDescriptor,
}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct NodeDescriptor {
    forward_link: u32,
    backward_link: u32,
    ty: NodeType,
    level: u8,
    #[brw(pad_after = 2)]
    record_count: u16,
}

#[derive(Derivative, Clone, Copy, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub enum NodeType {
    #[brw(magic = b"\x00")] Index,
    #[brw(magic = b"\x01")] Header,
    #[brw(magic = b"\x02")] Map,
    #[brw(magic = b"\xff")] Leaf,
}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct HeaderRecord {
    depth: u16,
    root: u32,
    leaf_count: u32,
    first_leaf: u32,
    last_leaf: u32,
    #[br(assert(node_size == 512))]
    node_size: u16,
    max_key_len: u16,
    node_count: u32,
    #[brw(pad_after = 76)]
    free_nodes: u32,
}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct MapRecord(
    #[derivative(Debug = "ignore")]
    [u8; 256]
);
