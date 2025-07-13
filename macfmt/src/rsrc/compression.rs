use binrw::{BinRead, BinResult};
use binrw::io::{Read, Seek};
use derivative::Derivative;

#[derive(Clone, Derivative, BinRead)]
#[derivative(Debug)]
#[brw(big)]
pub struct Header {
    sig: u32,
    hdrlen: u16,
    #[br(dbg)]
    version: u8,
    attrs: u8,
    biglen: u32,
    #[br(args(version))]
    variant: Variant,
}

#[derive(Clone, Derivative, BinRead)]
#[derivative(Debug)]
#[brw(big)]
#[br(import(version: u8))]
pub enum Variant {
    #[br(pre_assert(version == 0x08))]
    DonnBits {
        tab_ratio: u8,
        overrun: u8,
        alg_id: u16,
        tab_id: u16,
    },
}


impl Header {
    pub fn decompress<R: Read + Seek>(&self, data: &mut R) -> BinResult<Vec<u8>> {
        match self.variant {
            Variant::DonnBits { .. } => undonnbits(data, self.biglen as usize),
        }
    }
}

const DONN_LUT: [u16; 180] = [
    0x0000, 0x4EBA, 0x0008, 0x4E75, 0x000C, 0x4EAD, 0x2053, 0x2F0B,
    0x6100, 0x0010, 0x7000, 0x2F00, 0x486E, 0x2050, 0x206E, 0x2F2E,
    0xFFFC, 0x48E7, 0x3F3C, 0x0004, 0xFFF8, 0x2F0C, 0x2006, 0x4EED,
    0x4E56, 0x2068, 0x4E5E, 0x0001, 0x588F, 0x4FEF, 0x0002, 0x0018,
    0x6000, 0xFFFF, 0x508F, 0x4E90, 0x0006, 0x266E, 0x0014, 0xFFF4,
    0x4CEE, 0x000A, 0x000E, 0x41EE, 0x4CDF, 0x48C0, 0xFFF0, 0x2D40,
    0x0012, 0x302E, 0x7001, 0x2F28, 0x2054, 0x6700, 0x0020, 0x001C,
    0x205F, 0x1800, 0x266F, 0x4878, 0x0016, 0x41FA, 0x303C, 0x2840,
    0x7200, 0x286E, 0x200C, 0x6600, 0x206B, 0x2F07, 0x558F, 0x0028,
    0xFFFE, 0xFFEC, 0x22D8, 0x200B, 0x000F, 0x598F, 0x2F3C, 0xFF00,
    0x0118, 0x81E1, 0x4A00, 0x4EB0, 0xFFE8, 0x48C7, 0x0003, 0x0022,
    0x0007, 0x001A, 0x6706, 0x6708, 0x4EF9, 0x0024, 0x2078, 0x0800,
    0x6604, 0x002A, 0x4ED0, 0x3028, 0x265F, 0x6704, 0x0030, 0x43EE,
    0x3F00, 0x201F, 0x001E, 0xFFF6, 0x202E, 0x42A7, 0x2007, 0xFFFA,
    0x6002, 0x3D40, 0x0C40, 0x6606, 0x0026, 0x2D48, 0x2F01, 0x70FF,
    0x6004, 0x1880, 0x4A40, 0x0040, 0x002C, 0x2F08, 0x0011, 0xFFE4,
    0x2140, 0x2640, 0xFFF2, 0x426E, 0x4EB9, 0x3D7C, 0x0038, 0x000D,
    0x6006, 0x422E, 0x203C, 0x670C, 0x2D68, 0x6608, 0x4A2E, 0x4AAE,
    0x002E, 0x4840, 0x225F, 0x2200, 0x670A, 0x3007, 0x4267, 0x0032,
    0x2028, 0x0009, 0x487A, 0x0200, 0x2F2B, 0x0005, 0x226E, 0x6602,
    0xE580, 0x670E, 0x660A, 0x0050, 0x3E00, 0x660C, 0x2E00, 0xFFEE,
    0x206D, 0x2040, 0xFFE0, 0x5340, 0x6008, 0x0480, 0x0068, 0x0B7C,
    0x4400, 0x41E8, 0x4841, 0x0000,
];

fn varint<R: Read + Seek>(r: &mut R) -> BinResult<usize> {
    let mut val = [0];
    r.read_exact(&mut val)?;
    let val = val[0];
    match val {
        0..128 => Ok(val as usize),
        255 => {
            let mut int = [0; 4];
            r.read_exact(&mut int)?;
            Ok(u32::from_be_bytes(int) as usize)
        },
        _ => {
            let mut add = [0];
            r.read_exact(&mut add)?;
            println!("0x{:02x} 0x{:02x}", val, add[0]);
            let mut ret = 0;
            ret |= ((val as usize) << 12) & 0xf00;
            ret |= add[0] as usize;
            println!("{:x}", ret);
            Ok(ret)
        },
    }
}

fn undonnbits<R: Read + Seek>(r: &mut R, len: usize) -> BinResult<Vec<u8>> {
    let mut ret = Vec::with_capacity(len as usize);
    let mut var_tab: Vec<Vec<u8>> = Vec::new();

    loop {
        let mut op: [u8; 1] = [0];
        println!("@{:x}", r.stream_position()?);
        r.read_exact(&mut op)?;
        let op: u8 = op[0];
        println!("op {:x}", op);
        match op {
            0x00..0x20 => {
                let save = op >= 0x10;
                let len = if op == 0x00 || op == 0x10 {
                    let mut len_byte = [0];
                    r.read_exact(&mut len_byte)?;
                    len_byte[0] as usize * 2
                } else {
                    (op & 0x0f) as usize * 2
                };
                let mut tmp = vec![0; len];
                r.read_exact(&mut tmp)?;
                ret.extend(&tmp);
                if save {
                    var_tab.push(tmp);
                }
            },
            0x20 | 0x21 => {
                let mut tmp = [0];
                r.read_exact(&mut tmp)?;
                let mut idx = tmp[0] as usize;
                let mut idx = 0x28 + ((op as usize & 0xf) << 8) | idx;
                ret.extend(var_tab.get(idx).unwrap());
            },
            0x22 => {
                let mut bytes = [0; 2];
                r.read_exact(&mut bytes)?;
                let idx = u16::from_be_bytes(bytes) + 0x28;
                ret.extend(var_tab.get(idx as usize).unwrap());
            },
            0x23..0x4b => ret.extend(var_tab.get((op - 0x23) as usize).unwrap()),
            0x4b..0xfe => ret.extend(DONN_LUT[op as usize - 0x4b].to_be_bytes()),
            0xfe => {
                let mut tmp = [0];
                r.read_exact(&mut tmp)?;
                let extop = tmp[0];
                match extop {
                    0x00 => {
                        let segment = (varint(r)? as u16).to_be_bytes();
                        let count = varint(r)?;
                        let mut offset: u16 = 0;
                        for _ in 0..count {
                            let diff = varint(r)? as u16;
                            offset = offset.wrapping_add(diff).wrapping_sub(6);
                            ret.extend(offset.to_be_bytes());
                            ret.extend([0x3f, 0x3c]);
                            ret.extend(&segment);
                            ret.extend([0xa9, 0xf0]);
                        }
                    },
                    0x02 | 0x03 => {
                        let val = match extop {
                            0x02 => (varint(r)? as u8).to_be_bytes().to_vec(),
                            0x03 => (varint(r)? as u16).to_be_bytes().to_vec(),
                            _ => unreachable!(),
                        };
                        let count = varint(r)? + 1;
                        for _ in 0..count {
                            ret.extend(&val);
                        }
                    },
                    _ => todo!("donnbits ext op 0x{:02x}", extop),
                }
            },
            0xff => break,
            _ => todo!("donnbits token 0x{:02x}", op),
        }
    }

    Ok(ret)
}
