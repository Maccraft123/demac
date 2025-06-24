use std::str::Utf8Error;
use std::fmt;

use binrw::{BinRead, BinWrite};
use time::{OffsetDateTime, PrimitiveDateTime};
use deku::{
    DekuRead,
    reader::Reader, DekuReader, DekuError,
    ctx::Endian,
    no_std_io::{Read, Seek},
};

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

impl<const CAP: usize> DekuReader<'_, Endian> for PascalString<CAP> {
    fn from_reader_with_ctx<R: Read + Seek>(
        reader: &mut Reader<R>,
        _: Endian,
    ) -> Result<Self, DekuError> {
        let mut len: [u8; 1] = [0];
        reader.read_bytes_const(&mut len, deku::ctx::Order::Lsb0)?;

        let mut data: [u8; CAP] = [0u8; CAP];
        reader.read_bytes_const(&mut data, deku::ctx::Order::Lsb0)?;

        Ok(Self {
            len: len[0],
            data,
        })
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
