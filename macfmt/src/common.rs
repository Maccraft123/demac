use std::str::Utf8Error;
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

#[derive(Clone, BinRead, BinWrite)]
pub struct SizedString<const SIZE: usize> {
    data: [u8; SIZE],
}

impl<const CAP: usize> SizedString<CAP> {
    fn try_as_str(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(&self.data[..])
    }
}

impl<const CAP: usize> fmt::Debug for SizedString<CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.try_as_str() {
            Ok(s) => write!(f, "SizedString(\"{s}\")"),
            Err(_) => write!(f, "SizedString({:x?})", &self.data),
        }
    }
}

#[derive(Clone, BinRead, BinWrite)]
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

#[derive(Clone, BinRead, BinWrite)]
pub struct DynamicPascalString {
    len: u8,
    #[br(count = len)]
    data: Vec<u8>,
}

impl DynamicPascalString {
    fn try_as_str(&self) -> Result<&str, Utf8Error> {
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

#[derive(Clone, Copy, DekuRead, BinRead, BinWrite)]
#[brw(big)]
#[deku(endian = "big", ctx = "_: Endian")]
#[repr(transparent)]
pub struct DateTime(u32);

use time::UtcOffset;
impl DateTime {
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
