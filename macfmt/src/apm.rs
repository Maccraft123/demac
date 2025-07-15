use std::fmt;

use binrw::io::{Read, Seek, SeekFrom};
use binrw::{BinRead, BinResult, NullString, binread};
use derivative::Derivative;

use crate::fs::hfs::Hfs;

pub struct ApmDrive<'a, R: Read + Seek> {
    table: ApmTable,
    reader: &'a mut R,
}

impl<'a, R: Read + Seek> ApmDrive<'a, R> {
    pub fn block_size(&self) -> u16 {
        self.table.driver_descriptor.block_size
    }
    pub fn new(reader: &'a mut R) -> BinResult<Self> {
        let table = ApmTable::read(reader)?;
        Ok(Self { table, reader })
    }
    pub fn partitions(&self) -> &[Partition] {
        &self.table.partitions
    }
    pub fn drivers(&self) -> &[Driver] {
        &self.table.driver_descriptor.drivers
    }
    pub fn partition_hfs(&mut self, _p: &Partition) -> BinResult<Hfs> {
        todo!()
    }
    pub fn read_partition_data(&mut self, p: &Partition) -> BinResult<Vec<u8>> {
        let read_start = (p.start + p.data_start) * self.block_size() as u32;
        let read_size = p.data_size as usize;
        let mut buf: Vec<u8> = vec![0; read_size];
        self.reader.seek(SeekFrom::Start(read_start as u64))?;
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }
}

#[binread]
#[derive(Derivative)]
#[derivative(Debug)]
#[brw(big)]
pub struct ApmTable {
    #[br(pad_size_to = 512)]
    driver_descriptor: Block0,
    #[br(temp, restore_position)]
    first_partition: Partition,
    #[br(count = first_partition.partition_count)]
    partitions: Vec<Partition>,
    #[br(count = apm_pad_size(&partitions))]
    #[derivative(Debug = "ignore")]
    _pad: Vec<[u8; 512]>,
}

fn apm_pad_size(partitions: &[Partition]) -> u32 {
    partitions
        .iter()
        .find(|entry| entry.kind == PartitionType::ApplePartitionMap)
        .map(|entry| entry.size - entry.partition_count)
        .unwrap_or(0)
}

#[derive(BinRead, Derivative)]
#[derivative(Debug)]
#[brw(big, magic = b"ER")]
pub struct Block0 {
    block_size: u16,
    block_count: u32,
    _dev_type: u16,
    _dev_id: u16,
    _sb_data: u32,
    driver_count: u16,
    #[br(count = driver_count)]
    drivers: Vec<Driver>,
}

#[derive(BinRead, Derivative)]
#[derivative(Debug)]
#[brw(big)]
pub struct Driver {
    start: u32,
    size: u16,
    #[br(parse_with = os_type_parser)]
    os_type: OsType,
}

impl Driver {
    pub fn start(&self) -> u32 {
        self.start
    }
    pub fn size(&self) -> u16 {
        self.size
    }
    pub fn os_type(&self) -> OsType {
        self.os_type
    }
}

#[derive(Debug, Copy, Clone)]
pub enum OsType {
    MacOs,
    Other(u16),
}

impl fmt::Display for OsType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OsType::MacOs => write!(f, "MacOS"),
            OsType::Other(ty) => write!(f, "<0x{:x}>", ty),
        }
    }
}

#[binrw::parser(reader)]
fn os_type_parser() -> BinResult<OsType> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    match u16::from_be_bytes(buf) {
        0x0001 => Ok(OsType::MacOs),
        other => Ok(OsType::Other(other)),
    }
}

#[derive(BinRead, Derivative)]
#[derivative(Debug)]
#[brw(big, magic = b"PM")]
pub struct Partition {
    #[derivative(Debug = "ignore")]
    _pad: u16,
    partition_count: u32,
    start: u32,
    size: u32,
    #[brw(pad_size_to = 32)]
    name: NullString,
    #[br(parse_with = partition_type_parser)]
    kind: PartitionType,
    data_start: u32,
    data_size: u32,
    status: u32,
    boot_start: u32,
    boot_size: u32,
    boot_load_addr: u32,
    #[derivative(Debug = "ignore")]
    _pad2: u32,
    boot_entry: u32,
    #[derivative(Debug = "ignore")]
    _pad3: u32,
    boot_checksum: u32,
    #[br(parse_with = processor_type_parser)]
    proc_type: ProcessorType,
    #[derivative(Debug = "ignore")]
    _pad4: [u16; 188],
}

impl Partition {
    pub fn start(&self) -> u32 {
        self.start
    }
    pub fn size(&self) -> u32 {
        self.size
    }
    pub fn name(&self) -> Result<&str, std::str::Utf8Error> {
        str::from_utf8(&self.name)
    }
    pub fn kind(&self) -> &PartitionType {
        &self.kind
    }
}

#[binrw::parser(reader)]
fn partition_type_parser() -> BinResult<PartitionType> {
    let mut buf = [0u8; 32];
    reader.read_exact(&mut buf)?;
    match str::from_utf8(&buf) {
        Ok(s) => match s.trim_end_matches('\0') {
            "Apple_partition_map" => Ok(PartitionType::ApplePartitionMap),
            "Apple_Driver" => Ok(PartitionType::AppleDriver),
            "Apple_Driver43" => Ok(PartitionType::AppleDriver43),
            "Apple_MFS" => Ok(PartitionType::AppleMfs),
            "Apple_HFS" => Ok(PartitionType::AppleHfs),
            "Apple_Unix_SVR2" => Ok(PartitionType::AppleUnixSvr2),
            "Apple_PRODOS" => Ok(PartitionType::AppleProDos),
            "Apple_Free" => Ok(PartitionType::AppleFree),
            "Apple_Scratch" => Ok(PartitionType::AppleScratch),
            "Apple_Bootstrap" => Ok(PartitionType::AppleBootstrap),
            "Linux" => Ok(PartitionType::Linux),
            "Linux_RAID" => Ok(PartitionType::LinuxRaid),
            "Linux_swap" => Ok(PartitionType::LinuxSwap),
            _ => Ok(PartitionType::Other(s.to_string())),
        },
        Err(_) => Ok(PartitionType::NonUtf8(buf)),
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PartitionType {
    ApplePartitionMap,
    AppleDriver,
    AppleDriver43,
    AppleMfs,
    AppleHfs,
    AppleUnixSvr2,
    AppleProDos,
    AppleFree,
    AppleScratch,
    AppleBootstrap,
    Linux,
    LinuxRaid,
    LinuxSwap,
    Other(String),
    NonUtf8([u8; 32]),
}

impl fmt::Display for PartitionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PartitionType::ApplePartitionMap => write!(f, "Partition map"),
            PartitionType::AppleDriver => write!(f, "Device driver"),
            PartitionType::AppleDriver43 => write!(f, "SCSI Manager 4.3 device driver"),
            PartitionType::AppleMfs => write!(f, "MFS"),
            PartitionType::AppleHfs => write!(f, "HFS"),
            PartitionType::AppleUnixSvr2 => write!(f, "Unix"),
            PartitionType::AppleProDos => write!(f, "ProDOS"),
            PartitionType::AppleFree => write!(f, "<unused>"),
            PartitionType::AppleScratch => write!(f, "<empty>"),
            PartitionType::AppleBootstrap => write!(f, "Bootstrap"),
            PartitionType::Linux => write!(f, "Linux"),
            PartitionType::LinuxRaid => write!(f, "Linux RAID"),
            PartitionType::LinuxSwap => write!(f, "Linux Swap"),
            PartitionType::Other(ty) => write!(f, "{}", ty),
            PartitionType::NonUtf8(ty) => write!(f, "{:x?}", ty),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ProcessorType {
    M68000,
    M68008,
    M68010,
    M68012,
    M68020,
    M68030,
    M68040,
    PowerPc,
    Unspecified,
    Other(String),
    NonUtf8([u8; 16]),
}

#[binrw::parser(reader)]
fn processor_type_parser() -> BinResult<ProcessorType> {
    let mut buf = [0u8; 16];
    reader.read_exact(&mut buf)?;
    match str::from_utf8(&buf) {
        Ok(s) => match s.trim_end_matches('\0') {
            "68000" => Ok(ProcessorType::M68000),
            "68008" => Ok(ProcessorType::M68008),
            "68010" => Ok(ProcessorType::M68010),
            "68012" => Ok(ProcessorType::M68012),
            "68020" => Ok(ProcessorType::M68020),
            "68030" => Ok(ProcessorType::M68030),
            "68040" => Ok(ProcessorType::M68040),
            "powerpc" => Ok(ProcessorType::PowerPc),
            "" => Ok(ProcessorType::Unspecified),
            _ => Ok(ProcessorType::Other(s.to_string())),
        },
        Err(_) => Ok(ProcessorType::NonUtf8(buf)),
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use crate::apm::{ApmDrive, PartitionType};

    use flate2::read::GzDecoder;

    static HD_100MB: &'static [u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/100mb-hfs.hda.gz"
    ));
    #[test]
    fn decode() {
        let mut decoder = GzDecoder::new(HD_100MB);
        let mut vec = Vec::new();
        decoder.read_to_end(&mut vec).unwrap();
        let mut bytes = Cursor::new(vec);
        let disk = super::ApmDrive::new(&mut bytes).unwrap();
        let expected_partitions = [
            (PartitionType::ApplePartitionMap, 1, 63),
            (PartitionType::AppleDriver43, 64, 32),
            (PartitionType::AppleHfs, 96, 184224),
        ];

        let got_partitions: Vec<(PartitionType, u32, u32)> = disk
            .partitions()
            .iter()
            .map(|p| (p.kind().clone(), p.start(), p.size()))
            .collect();

        assert_eq!(&got_partitions, &expected_partitions);
    }
}
