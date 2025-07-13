use std::fmt;
use std::str::Utf8Error;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::i18n::{MacRoman, MacScript};
use binrw::{BinRead, BinResult, BinWrite};
use bitfield_struct::bitfield;
use time::OffsetDateTime;

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
    #[br(count = len, map = |buf: Vec<u8>| buf.into_iter().map(|b| MacRoman::decode(b)).collect())]
    #[bw(try_map = |s: &String| s.chars().map(|v| MacRoman::encode(v)).collect::<Result<Vec<u8>, _>>())]
    data: String,
}

impl DynamicPascalString {
    pub fn new(t: impl Into<String>) -> Self {
        let data: String = t.into();
        Self {
            len: data.len() as u8,
            data,
        }
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn as_str(&self) -> &str {
        self.data.as_str()
    }
    pub fn try_as_str(&self) -> Result<&str, std::convert::Infallible> {
        Ok(self.data.as_str())
    }
    pub fn as_mut(&mut self) -> &mut String {
        &mut self.data
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
        Self { data }
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

#[derive(Clone, Copy, Eq, PartialEq, BinRead, BinWrite)]
#[brw(big)]
#[repr(transparent)]
pub struct DateTime2k(u32);

use time::UtcOffset;
impl DateTime2k {
    const OFFSET_FROM_UNIX_EPOCH: u32 = 946684800;
    pub fn to_system_time(self) -> SystemTime {
        UNIX_EPOCH
            .checked_add(std::time::Duration::from_secs((946684800 + self.0) as u64))
            .unwrap()
    }
    pub fn now() -> Self {
        DateTime2k(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32
                - Self::OFFSET_FROM_UNIX_EPOCH,
        )
    }
    fn epoch_start() -> OffsetDateTime {
        time::macros::datetime!(2000-01-01 00:00).assume_offset(UtcOffset::UTC)
    }
}

impl From<&DateTime2k> for OffsetDateTime {
    fn from(t: &DateTime2k) -> OffsetDateTime {
        DateTime2k::epoch_start() + std::time::Duration::from_secs(t.0 as u64)
    }
}

impl From<DateTime2k> for OffsetDateTime {
    fn from(t: DateTime2k) -> OffsetDateTime {
        DateTime2k::epoch_start() + std::time::Duration::from_secs(t.0 as u64)
    }
}

impl fmt::Debug for DateTime2k {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let tmp: OffsetDateTime = self.into();
        write!(f, "DateTime2k(\"{:?}\")", tmp)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, BinRead, BinWrite)]
#[brw(big)]
#[repr(transparent)]
pub struct DateTime(u32);

impl DateTime {
    const OFFSET_FROM_UNIX_EPOCH: u32 = 2082844800;
    pub fn to_system_time(self) -> SystemTime {
        UNIX_EPOCH
            .checked_add(std::time::Duration::from_secs(
                (self.0 - Self::OFFSET_FROM_UNIX_EPOCH) as u64,
            ))
            .unwrap()
    }
    pub fn now() -> Self {
        DateTime(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32
                + Self::OFFSET_FROM_UNIX_EPOCH,
        )
    }
    fn epoch_start() -> OffsetDateTime {
        time::macros::datetime!(1904-01-01 00:00).assume_offset(UtcOffset::UTC)
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, BinRead, BinWrite)]
#[brw(big)]
pub struct Point {
    pub y: i16,
    pub x: i16,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, BinRead, BinWrite)]
#[brw(big)]
pub struct Rect {
    pub top_left: Point,
    pub bottom_right: Point,
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

#[bitfield(u8)]
pub struct Style {
    bold: bool,
    italic: bool,
    underline: bool,
    outline: bool,
    shadow: bool,
    condense: bool,
    extend: bool,
    __: bool,
}

impl BinRead for Style {
    type Args<'a> = ();
    fn read_options<R: binrw::io::Read + binrw::io::Seek>(
        reader: &mut R,
        _: binrw::Endian,
        _: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        Ok(Self::from(buf[0]))
    }
}

impl BinWrite for Style {
    type Args<'a> = ();
    fn write_options<W: binrw::io::Write + binrw::io::Seek>(
        &self,
        writer: &mut W,
        _: binrw::Endian,
        _: Self::Args<'_>,
    ) -> BinResult<()> {
        let buf = [self.into_bits()];
        writer.write_all(&buf)?;
        Ok(())
    }
}
