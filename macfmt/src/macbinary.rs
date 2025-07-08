use binrw::{BinRead, BinWrite};
use crc::Crc;
use derivative::Derivative;

use crate::common::{DateTime, FinderInfo, PascalString};

pub fn is_macbinary2(data: &[u8]) -> bool {
    data.len() > 126 && checksum_matches(&data[..=124], u16::from_be_bytes([data[125], data[126]]))
}

fn checksum_matches(data: &[u8], checksum: u16) -> bool {
    let crc = Crc::<u16>::new(&crc::CRC_16_XMODEM);
    let calc_sum = crc.checksum(data);
    calc_sum == checksum
}

#[binrw::binread]
#[binrw::binwrite]
#[derive(Clone, Derivative)]
#[derivative(Debug)]
#[brw(big)]
#[br(assert(is_macbinary2(&header_data)))]
pub struct MacBinary2 {
    #[br(temp, restore_position)]
    #[bw(ignore)]
    header_data: [u8; 128],
    #[brw(align_after = 128)]
    header: MacBinary2Header,
    #[br(count = header.data_fork_size, align_after = 128)]
    #[derivative(Debug = "ignore")]
    data: Vec<u8>,
    #[br(count = header.resource_fork_size)]
    #[derivative(Debug = "ignore")]
    resource: Vec<u8>,
}

impl MacBinary2 {
    pub fn resource_fork(&self) -> &[u8] {
        &self.resource
    }
}

#[derive(Clone, Derivative, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct MacBinary2Header {
    version: u8,
    filename: PascalString<63>,
    finder_info: FinderInfo,
    #[brw(pad_after = 1)]
    protected: u8,
    data_fork_size: u32,
    resource_fork_size: u32,
    creation_time: DateTime,
    modification_time: DateTime,
    get_info_comment_len: u16,
    #[brw(pad_after = 14)]
    low_byte_of_finder_flags: u8,
    unpacked_len: u32,
    secondary_header_len: u16,
    uploaders_macbinary_ii_version: u8,
    minimum_macbinary_ii_version: u8,
    // this field wasn't documented anywhere
    // i don't know why crc matches
    // i don't want to know
    what: u8,
    crc: u16,
}
