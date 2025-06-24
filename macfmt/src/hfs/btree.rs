use std::{fmt, mem};

use binrw::{BinRead, BinWrite};
use binrw::io::SeekFrom;
use crate::common::{DateTime, PascalString};
use derivative::Derivative;
use static_assertions::assert_eq_size;

#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(big)]
pub struct NodeDescriptor {
    forward_link: u32,
    backward_link: u32,
    node_type: NodeType,
    node_depth: u8,
    #[brw(pad_after = 2)]
    record_count: u16,
}

#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(repr = u8)]
pub enum NodeType {
    Index = 0x00,
    Header = 0x01,
    Map = 0x02,
    Leaf = 0xff,
}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct Node {
    desc: NodeDescriptor,
    #[br(count = desc.record_count)]
    records: Vec<Record>,

    #[br(align_after = 512)]
    e: u16,
}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct Record {
    key_len: u8,
    #[br(count = (key_len))]
    key: Vec<u8>,
}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct HeaderNode {
    desc: NodeDescriptor,
    hdr_rec: HeaderRecord,
    #[derivative(Debug = "ignore")]
    unused: [u8; 128],
    map_rec: MapRecord,
    empty_space_off: u16,
    record_2_off: u16,
    record_1_off: u16,
    record_0_off: u16,
    #[br(count = hdr_rec.node_count-1)]
    nodes: Vec<Node>,
}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
pub struct HeaderRecord {
    depth: u16,
    root: u32,
    leaf_count: u32,
    first_leaf: u32,
    last_leaf: u32,
    node_size: u16,
    key_len: u16,
    node_count: u32,
    #[brw(pad_after = 76)]
    free_node_count: u32,
}

#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
pub struct MapRecord {
    #[derivative(Debug = "ignore")]
    bitmap: [u8; 256],
}


#[derive(Derivative, Clone, BinRead, BinWrite)]
#[derivative(Debug)]
pub struct IndexRecord {
}
