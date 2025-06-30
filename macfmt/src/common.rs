use std::str::Utf8Error;
use std::time::{UNIX_EPOCH, SystemTime};
use std::fmt;

use binrw::{BinRead, BinWrite};
use time::{OffsetDateTime, PrimitiveDateTime};
use derivative::Derivative;
use deku::{
    DekuRead,
    reader::Reader, DekuReader, DekuError,
    ctx::Endian,
    no_std_io::{Read, Seek},
};

#[derive(Clone, Eq, PartialEq, BinRead, BinWrite)]
pub struct SizedString<const SIZE: usize> {
    data: [u8; SIZE],
}

impl<const SIZE: usize> SizedString<SIZE> {
    pub fn new(data: [u8; SIZE]) -> Self {
        Self { data }
    }
    pub fn as_inner(&self) -> &[u8; SIZE] {
        &self.data
    }
    pub fn try_as_str(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(&self.data[..])
    }
}

impl<const SIZE: usize> From<[u8; SIZE]> for SizedString<SIZE> {
    fn from(data: [u8; SIZE]) -> SizedString<SIZE> {
        Self { data }
    }
}

impl<const SIZE: usize> fmt::Debug for SizedString<SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.try_as_str() {
            Ok(s) => write!(f, "SizedString(\"{s}\")"),
            Err(_) => write!(f, "SizedString({:x?})", &self.data),
        }
    }
}

#[derive(Clone, BinRead, BinWrite, Eq, PartialEq)]
pub struct PascalString<const CAP: usize> {
    len: u8,
    data: [u8; CAP],
}

impl<const CAP: usize> PascalString<CAP> {
    fn try_as_str(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(&self.data[..(self.len as usize)])
    }
}

impl<const CAP: usize> fmt::Debug for PascalString<CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.try_as_str() {
            Ok(s) => write!(f, "PascalString(\"{s}\")"),
            Err(_) => write!(f, "PascalString({:x?})", &self.data),
        }
    }
}

#[derive(Clone, BinRead, BinWrite, Eq, PartialEq)]
#[brw(big)]
pub struct DynamicPascalString {
    len: u8,
    #[br(count = len)]
    data: Vec<u8>,
}

impl DynamicPascalString {
    pub fn new(t: impl Into<String>) -> Self {
        let string: String = t.into();
        let data = string.into_bytes();
        Self {
            len: data.len() as u8,
            data,
        }
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn try_as_str(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(&self.data[..(self.len as usize)])
    }
}


impl fmt::Debug for DynamicPascalString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.try_as_str() {
            Ok(s) => write!(f, "DynamicPascalString(\"{s}\")"),
            Err(_) => write!(f, "DynamicPascalString({:x?})", &self.data),
        }
    }
}

#[derive(Clone, BinRead, BinWrite, Eq, PartialEq)]
#[brw(big)]
#[br(import { count: usize })]
pub struct UnsizedPascalString {
    #[br(count = count)]
    data: Vec<u8>,
}

impl UnsizedPascalString {
    pub fn new(t: impl Into<String>) -> Self {
        let string: String = t.into();
        let data = string.into_bytes();
        Self {
            data,
        }
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn try_as_str(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(&self.data[..])
    }
}


impl fmt::Debug for UnsizedPascalString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.try_as_str() {
            Ok(s) => write!(f, "UnsizedPascalString(\"{s}\")"),
            Err(_) => write!(f, "UnsizedPascalString({:x?})", &self.data),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, DekuRead, BinRead, BinWrite)]
#[brw(big)]
#[deku(endian = "big", ctx = "_: Endian")]
#[repr(transparent)]
pub struct DateTime(u32);


use time::UtcOffset;
impl DateTime {
    const OFFSET_FROM_UNIX_EPOCH: u32 = 2082844800;
    pub fn to_system_time(self) -> SystemTime {
        UNIX_EPOCH
            .checked_add(std::time::Duration::from_secs((self.0 - Self::OFFSET_FROM_UNIX_EPOCH) as u64))
            .unwrap()
    }
    pub fn now() -> Self {
        DateTime(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            as u32
            + Self::OFFSET_FROM_UNIX_EPOCH)
    }
    fn epoch_start() -> OffsetDateTime {
        time::macros::datetime!(1904-01-01 00:00)
            .assume_offset(
                UtcOffset::UTC
            )
    }
    fn to_unix_timestamp(self) -> i64 {
        let tmp: OffsetDateTime = self.into();
        tmp.unix_timestamp()
    }
}

impl From<&DateTime> for OffsetDateTime {
    fn from(t: &DateTime) -> OffsetDateTime {
        DateTime::epoch_start() + std::time::Duration::from_secs(t.0 as u64)
    }
}

impl From<DateTime> for OffsetDateTime {
    fn from(t: DateTime) -> OffsetDateTime {
        DateTime::epoch_start() + std::time::Duration::from_secs(t.0 as u64)
    }
}

impl fmt::Debug for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let tmp: OffsetDateTime = self.into();
        write!(f, "DateTime(\"{:?}\")", tmp)
    }
}


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
    #[derivative(Debug(format_with="BootBlocks::code_vec_fmt"))]
    #[br(count = 2)]
    code: Vec<u16>,
}

impl BootBlocks {
    fn code_vec_fmt(v: &Vec<u16>, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
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

#[derive(Debug, Clone, Eq, PartialEq, BinRead, BinWrite)]
#[brw(big)]
pub struct Point {
    y: i16,
    x: i16,
}

#[derive(Debug, Clone, Eq, PartialEq, BinRead, BinWrite)]
#[brw(big)]
pub struct Rect {
    top_left: Point,
    bottom_right: Point,
}

#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(big)]
pub struct FinderInfo {
    file_type: SizedString<4>,
    file_creator: SizedString<4>,
    flags: u16,
    location: Point,
    parent_dir: u16,
}

#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(big)]
pub struct ExtraFinderInfo {
    #[brw(pad_after = 6)]
    icon_id: u16,
    script: u8,
    flags: u8,
    comment_id: u16,
    home_dir_id: u32,
}
