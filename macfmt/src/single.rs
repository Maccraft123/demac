use std::{fmt, mem};

use crate::common::{DateTime2k, PascalString, UnsizedPascalString};
use crate::rsrc::Resource;
use binrw::io::SeekFrom;
use binrw::{BinRead, BinResult, BinWrite};
use derivative::Derivative;

#[derive(Derivative, Clone, BinRead)]
#[derivative(Debug)]
#[brw(big)]
pub enum AppleFile {
    #[brw(magic = b"\x00\x05\x16\x07")]
    Double(Header),
    #[brw(magic = b"\x00\x05\x16\x00")]
    Single(Header),
}

impl AppleFile {
    fn header(&self) -> &Header {
        match self {
            AppleFile::Single(hdr) => hdr,
            AppleFile::Double(hdr) => hdr,
        }
    }
    pub fn entries(&self) -> impl Iterator<Item = &EntryData> {
        self.header().entries()
    }
    pub fn resource_fork(&self) -> Option<&[u8]> {
        for entry in self.header().entries() {
            if let EntryData::ResourceFork(vec) = entry {
                return Some(vec);
            }
        }

        None
    }
}

#[derive(Derivative, Clone, BinRead)]
#[derivative(Debug)]
#[brw(big)]
pub struct Header {
    #[brw(pad_after = 16)]
    version: u32,
    entry_count: u16,
    #[br(count = entry_count)]
    entries: Vec<Entry>,
}

impl Header {
    pub fn entries(&self) -> impl Iterator<Item = &EntryData> {
        self.entries.iter().map(|v| &v.data)
    }
}

#[derive(Derivative, Clone, BinRead)]
#[derivative(Debug)]
#[brw(big)]
pub struct Entry {
    id: u32,
    offset: u32,
    length: u32,
    #[br(seek_before = SeekFrom::Start(offset as u64), restore_position)]
    //#[bw(seek_before = SeekFrom::Start(*offset as u64), restore_position)]
    #[br(args { id, length })]
    data: EntryData,
}

#[derive(Derivative, Clone, BinRead)]
#[derivative(Debug)]
#[brw(big)]
#[br(import { id: u32, length: u32 })]
pub enum EntryData {
    #[br(pre_assert(id == 1))]
    DataFork(
        #[br(count = length)]
        #[derivative(Debug = "ignore")]
        Vec<u8>,
    ),
    #[br(pre_assert(id == 2))]
    ResourceFork(#[br(count = length)] Vec<u8>),
    #[br(pre_assert(id == 3))]
    FileName(#[br(count = length)] UnsizedPascalString),
    #[br(pre_assert(id == 4))]
    MacComment(#[br(count = length)] Vec<u8>),
    #[br(pre_assert(id == 5))]
    MacIcon(#[br(count = length)] Vec<u8>),
    #[br(pre_assert(id == 6))]
    MacColorIcon(#[br(count = length)] Vec<u8>),
    #[br(pre_assert(id == 8))]
    FileDates {
        creation: u32,
        modification: u32,
        backup: u32,
        access: u32,
    },
    #[br(pre_assert(id == 9))]
    FinderInfo(crate::common::FinderInfo, crate::common::ExtraFinderInfo),
    #[br(pre_assert(id == 10))]
    MacFileInfo(#[br(count = length)] Vec<u8>),
    #[br(pre_assert(id == 11))]
    ProdosFileInfo {
        access: u16,
        file_type: u16,
        aux_type: u32,
    },
    #[br(pre_assert(id == 12))]
    MsDosFileInfo { attrs: u16 },
    #[br(pre_assert(id == 13))]
    AfpShortName(#[br(count = length)] Vec<u8>),
    #[br(pre_assert(id == 14))]
    AfpFileInfo(#[br(count = length)] Vec<u8>),
    #[br(pre_assert(id == 15))]
    AfpDirId(#[br(count = length)] Vec<u8>),
    Other {
        #[br(calc = id)]
        id: u32,
        #[br(count = length)]
        #[derivative(Debug = "ignore")]
        data: Vec<u8>,
    },
}

#[binrw::parser(reader)]
fn parse_resources() -> BinResult<Vec<Resource>> {
    Resource::read(reader)
}
