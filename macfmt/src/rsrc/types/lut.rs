pub const fn lut4(v: u8) -> image::Rgb<u8> {
    match v {
        0x0 => image::Rgb([0xff, 0xff, 0xff]),
        0x1 => image::Rgb([0xfc, 0xf3, 0x03]),
        0x2 => image::Rgb([0xff, 0x64, 0x02]),
        0x3 => image::Rgb([0xdd, 0x08, 0x06]),
        0x4 => image::Rgb([0xf2, 0x08, 0x84]),
        0x5 => image::Rgb([0x46, 0x00, 0xa5]),
        0x6 => image::Rgb([0x00, 0x00, 0xd4]),
        0x7 => image::Rgb([0x02, 0xab, 0xea]),
        0x8 => image::Rgb([0x1f, 0xb7, 0x14]),
        0x9 => image::Rgb([0x00, 0x64, 0x11]),
        0xa => image::Rgb([0x56, 0x2c, 0x05]),
        0xb => image::Rgb([0x90, 0x71, 0x3a]),
        0xc => image::Rgb([0xc0, 0xc0, 0xc0]),
        0xd => image::Rgb([0x80, 0x80, 0x80]),
        0xe => image::Rgb([0x40, 0x40, 0x40]),
        0xf => image::Rgb([0x00, 0x00, 0x00]),
        _ => unreachable!(),
    }
}

pub const fn lut8(v: u8) -> image::Rgb<u8> {
    match v {
        0x00 => image::Rgb([0xff, 0xff, 0xff]),
        0x01 => image::Rgb([0xff, 0xff, 0xcc]),
        0x02 => image::Rgb([0xff, 0xff, 0x99]),
        0x03 => image::Rgb([0xff, 0xff, 0x66]),
        0x04 => image::Rgb([0xff, 0xff, 0x33]),
        0x05 => image::Rgb([0xff, 0xff, 0x00]),
        0x06 => image::Rgb([0xff, 0xcc, 0xff]),
        0x07 => image::Rgb([0xff, 0xcc, 0xcc]),
        0x08 => image::Rgb([0xff, 0xcc, 0x99]),
        0x09 => image::Rgb([0xff, 0xcc, 0x66]),
        0x0a => image::Rgb([0xff, 0xcc, 0x33]),
        0x0b => image::Rgb([0xff, 0xcc, 0x00]),
        0x0c => image::Rgb([0xff, 0x99, 0xff]),
        0x0d => image::Rgb([0xff, 0x99, 0xcc]),
        0x0e => image::Rgb([0xff, 0x99, 0x99]),
        0x0f => image::Rgb([0xff, 0x99, 0x66]),
        0x10 => image::Rgb([0xff, 0x99, 0x33]),
        0x11 => image::Rgb([0xff, 0x99, 0x00]),
        0x12 => image::Rgb([0xff, 0x66, 0xff]),
        0x13 => image::Rgb([0xff, 0x66, 0xcc]),
        0x14 => image::Rgb([0xff, 0x66, 0x99]),
        0x15 => image::Rgb([0xff, 0x66, 0x66]),
        0x16 => image::Rgb([0xff, 0x66, 0x33]),
        0x17 => image::Rgb([0xff, 0x66, 0x00]),
        0x18 => image::Rgb([0xff, 0x33, 0xff]),
        0x19 => image::Rgb([0xff, 0x33, 0xcc]),
        0x1a => image::Rgb([0xff, 0x33, 0x99]),
        0x1b => image::Rgb([0xff, 0x33, 0x66]),
        0x1c => image::Rgb([0xff, 0x33, 0x33]),
        0x1d => image::Rgb([0xff, 0x33, 0x00]),
        0x1e => image::Rgb([0xff, 0x00, 0xff]),
        0x1f => image::Rgb([0xff, 0x00, 0xcc]),
        0x20 => image::Rgb([0xff, 0x00, 0x99]),
        0x21 => image::Rgb([0xff, 0x00, 0x66]),
        0x22 => image::Rgb([0xff, 0x00, 0x33]),
        0x23 => image::Rgb([0xff, 0x00, 0x00]),
        0x24 => image::Rgb([0xcc, 0xff, 0xff]),
        0x25 => image::Rgb([0xcc, 0xff, 0xcc]),
        0x26 => image::Rgb([0xcc, 0xff, 0x99]),
        0x27 => image::Rgb([0xcc, 0xff, 0x66]),
        0x28 => image::Rgb([0xcc, 0xff, 0x33]),
        0x29 => image::Rgb([0xcc, 0xff, 0x00]),
        0x2a => image::Rgb([0xcc, 0xcc, 0xff]),
        0x2b => image::Rgb([0xcc, 0xcc, 0xcc]),
        0x2c => image::Rgb([0xcc, 0xcc, 0x99]),
        0x2d => image::Rgb([0xcc, 0xcc, 0x66]),
        0x2e => image::Rgb([0xcc, 0xcc, 0x33]),
        0x2f => image::Rgb([0xcc, 0xcc, 0x00]),
        0x30 => image::Rgb([0xcc, 0x99, 0xff]),
        0x31 => image::Rgb([0xcc, 0x99, 0xcc]),
        0x32 => image::Rgb([0xcc, 0x99, 0x99]),
        0x33 => image::Rgb([0xcc, 0x99, 0x66]),
        0x34 => image::Rgb([0xcc, 0x99, 0x33]),
        0x35 => image::Rgb([0xcc, 0x99, 0x00]),
        0x36 => image::Rgb([0xcc, 0x66, 0xff]),
        0x37 => image::Rgb([0xcc, 0x66, 0xcc]),
        0x38 => image::Rgb([0xcc, 0x66, 0x99]),
        0x39 => image::Rgb([0xcc, 0x66, 0x66]),
        0x3a => image::Rgb([0xcc, 0x66, 0x33]),
        0x3b => image::Rgb([0xcc, 0x66, 0x00]),
        0x3c => image::Rgb([0xcc, 0x33, 0xff]),
        0x3d => image::Rgb([0xcc, 0x33, 0xcc]),
        0x3e => image::Rgb([0xcc, 0x33, 0x99]),
        0x3f => image::Rgb([0xcc, 0x33, 0x66]),
        0x40 => image::Rgb([0xcc, 0x33, 0x33]),
        0x41 => image::Rgb([0xcc, 0x33, 0x00]),
        0x42 => image::Rgb([0xcc, 0x00, 0xff]),
        0x43 => image::Rgb([0xcc, 0x00, 0xcc]),
        0x44 => image::Rgb([0xcc, 0x00, 0x99]),
        0x45 => image::Rgb([0xcc, 0x00, 0x66]),
        0x46 => image::Rgb([0xcc, 0x00, 0x33]),
        0x47 => image::Rgb([0xcc, 0x00, 0x00]),
        0x48 => image::Rgb([0x99, 0xff, 0xff]),
        0x49 => image::Rgb([0x99, 0xff, 0xcc]),
        0x4a => image::Rgb([0x99, 0xff, 0x99]),
        0x4b => image::Rgb([0x99, 0xff, 0x66]),
        0x4c => image::Rgb([0x99, 0xff, 0x33]),
        0x4d => image::Rgb([0x99, 0xff, 0x00]),
        0x4e => image::Rgb([0x99, 0xcc, 0xff]),
        0x4f => image::Rgb([0x99, 0xcc, 0xcc]),
        0x50 => image::Rgb([0x99, 0xcc, 0x99]),
        0x51 => image::Rgb([0x99, 0xcc, 0x66]),
        0x52 => image::Rgb([0x99, 0xcc, 0x33]),
        0x53 => image::Rgb([0x99, 0xcc, 0x00]),
        0x54 => image::Rgb([0x99, 0x99, 0xff]),
        0x55 => image::Rgb([0x99, 0x99, 0xcc]),
        0x56 => image::Rgb([0x99, 0x99, 0x99]),
        0x57 => image::Rgb([0x99, 0x99, 0x66]),
        0x58 => image::Rgb([0x99, 0x99, 0x33]),
        0x59 => image::Rgb([0x99, 0x99, 0x00]),
        0x5a => image::Rgb([0x99, 0x66, 0xff]),
        0x5b => image::Rgb([0x99, 0x66, 0xcc]),
        0x5c => image::Rgb([0x99, 0x66, 0x99]),
        0x5d => image::Rgb([0x99, 0x66, 0x66]),
        0x5e => image::Rgb([0x99, 0x66, 0x33]),
        0x5f => image::Rgb([0x99, 0x66, 0x00]),
        0x60 => image::Rgb([0x99, 0x33, 0xff]),
        0x61 => image::Rgb([0x99, 0x33, 0xcc]),
        0x62 => image::Rgb([0x99, 0x33, 0x99]),
        0x63 => image::Rgb([0x99, 0x33, 0x66]),
        0x64 => image::Rgb([0x99, 0x33, 0x33]),
        0x65 => image::Rgb([0x99, 0x33, 0x00]),
        0x66 => image::Rgb([0x99, 0x00, 0xff]),
        0x67 => image::Rgb([0x99, 0x00, 0xcc]),
        0x68 => image::Rgb([0x99, 0x00, 0x99]),
        0x69 => image::Rgb([0x99, 0x00, 0x66]),
        0x6a => image::Rgb([0x99, 0x00, 0x33]),
        0x6b => image::Rgb([0x99, 0x00, 0x00]),
        0x6c => image::Rgb([0x66, 0xff, 0xff]),
        0x6d => image::Rgb([0x66, 0xff, 0xcc]),
        0x6e => image::Rgb([0x66, 0xff, 0x99]),
        0x6f => image::Rgb([0x66, 0xff, 0x66]),
        0x70 => image::Rgb([0x66, 0xff, 0x33]),
        0x71 => image::Rgb([0x66, 0xff, 0x00]),
        0x72 => image::Rgb([0x66, 0xcc, 0xff]),
        0x73 => image::Rgb([0x66, 0xcc, 0xcc]),
        0x74 => image::Rgb([0x66, 0xcc, 0x99]),
        0x75 => image::Rgb([0x66, 0xcc, 0x66]),
        0x76 => image::Rgb([0x66, 0xcc, 0x33]),
        0x77 => image::Rgb([0x66, 0xcc, 0x00]),
        0x78 => image::Rgb([0x66, 0x99, 0xff]),
        0x79 => image::Rgb([0x66, 0x99, 0xcc]),
        0x7a => image::Rgb([0x66, 0x99, 0x99]),
        0x7b => image::Rgb([0x66, 0x99, 0x66]),
        0x7c => image::Rgb([0x66, 0x99, 0x33]),
        0x7d => image::Rgb([0x66, 0x99, 0x00]),
        0x7e => image::Rgb([0x66, 0x66, 0xff]),
        0x7f => image::Rgb([0x66, 0x66, 0xcc]),
        0x80 => image::Rgb([0x66, 0x66, 0x99]),
        0x81 => image::Rgb([0x66, 0x66, 0x66]),
        0x82 => image::Rgb([0x66, 0x66, 0x33]),
        0x83 => image::Rgb([0x66, 0x66, 0x00]),
        0x84 => image::Rgb([0x66, 0x33, 0xff]),
        0x85 => image::Rgb([0x66, 0x33, 0xcc]),
        0x86 => image::Rgb([0x66, 0x33, 0x99]),
        0x87 => image::Rgb([0x66, 0x33, 0x66]),
        0x88 => image::Rgb([0x66, 0x33, 0x33]),
        0x89 => image::Rgb([0x66, 0x33, 0x00]),
        0x8a => image::Rgb([0x66, 0x00, 0xff]),
        0x8b => image::Rgb([0x66, 0x00, 0xcc]),
        0x8c => image::Rgb([0x66, 0x00, 0x99]),
        0x8d => image::Rgb([0x66, 0x00, 0x66]),
        0x8e => image::Rgb([0x66, 0x00, 0x33]),
        0x8f => image::Rgb([0x66, 0x00, 0x00]),
        0x90 => image::Rgb([0x33, 0xff, 0xff]),
        0x91 => image::Rgb([0x33, 0xff, 0xcc]),
        0x92 => image::Rgb([0x33, 0xff, 0x99]),
        0x93 => image::Rgb([0x33, 0xff, 0x66]),
        0x94 => image::Rgb([0x33, 0xff, 0x33]),
        0x95 => image::Rgb([0x33, 0xff, 0x00]),
        0x96 => image::Rgb([0x33, 0xcc, 0xff]),
        0x97 => image::Rgb([0x33, 0xcc, 0xcc]),
        0x98 => image::Rgb([0x33, 0xcc, 0x99]),
        0x99 => image::Rgb([0x33, 0xcc, 0x66]),
        0x9a => image::Rgb([0x33, 0xcc, 0x33]),
        0x9b => image::Rgb([0x33, 0xcc, 0x00]),
        0x9c => image::Rgb([0x33, 0x99, 0xff]),
        0x9d => image::Rgb([0x33, 0x99, 0xcc]),
        0x9e => image::Rgb([0x33, 0x99, 0x99]),
        0x9f => image::Rgb([0x33, 0x99, 0x66]),
        0xa0 => image::Rgb([0x33, 0x99, 0x33]),
        0xa1 => image::Rgb([0x33, 0x99, 0x00]),
        0xa2 => image::Rgb([0x33, 0x66, 0xff]),
        0xa3 => image::Rgb([0x33, 0x66, 0xcc]),
        0xa4 => image::Rgb([0x33, 0x66, 0x99]),
        0xa5 => image::Rgb([0x33, 0x66, 0x66]),
        0xa6 => image::Rgb([0x33, 0x66, 0x33]),
        0xa7 => image::Rgb([0x33, 0x66, 0x00]),
        0xa8 => image::Rgb([0x33, 0x33, 0xff]),
        0xa9 => image::Rgb([0x33, 0x33, 0xcc]),
        0xaa => image::Rgb([0x33, 0x33, 0x99]),
        0xab => image::Rgb([0x33, 0x33, 0x66]),
        0xac => image::Rgb([0x33, 0x33, 0x33]),
        0xad => image::Rgb([0x33, 0x33, 0x00]),
        0xae => image::Rgb([0x33, 0x00, 0xff]),
        0xaf => image::Rgb([0x33, 0x00, 0xcc]),
        0xb0 => image::Rgb([0x33, 0x00, 0x99]),
        0xb1 => image::Rgb([0x33, 0x00, 0x66]),
        0xb2 => image::Rgb([0x33, 0x00, 0x33]),
        0xb3 => image::Rgb([0x33, 0x00, 0x00]),
        0xb4 => image::Rgb([0x00, 0xff, 0xff]),
        0xb5 => image::Rgb([0x00, 0xff, 0xcc]),
        0xb6 => image::Rgb([0x00, 0xff, 0x99]),
        0xb7 => image::Rgb([0x00, 0xff, 0x66]),
        0xb8 => image::Rgb([0x00, 0xff, 0x33]),
        0xb9 => image::Rgb([0x00, 0xff, 0x00]),
        0xba => image::Rgb([0x00, 0xcc, 0xff]),
        0xbb => image::Rgb([0x00, 0xcc, 0xcc]),
        0xbc => image::Rgb([0x00, 0xcc, 0x99]),
        0xbd => image::Rgb([0x00, 0xcc, 0x66]),
        0xbe => image::Rgb([0x00, 0xcc, 0x33]),
        0xbf => image::Rgb([0x00, 0xcc, 0x00]),
        0xc0 => image::Rgb([0x00, 0x99, 0xff]),
        0xc1 => image::Rgb([0x00, 0x99, 0xcc]),
        0xc2 => image::Rgb([0x00, 0x99, 0x99]),
        0xc3 => image::Rgb([0x00, 0x99, 0x66]),
        0xc4 => image::Rgb([0x00, 0x99, 0x33]),
        0xc5 => image::Rgb([0x00, 0x99, 0x00]),
        0xc6 => image::Rgb([0x00, 0x66, 0xff]),
        0xc7 => image::Rgb([0x00, 0x66, 0xcc]),
        0xc8 => image::Rgb([0x00, 0x66, 0x99]),
        0xc9 => image::Rgb([0x00, 0x66, 0x66]),
        0xca => image::Rgb([0x00, 0x66, 0x33]),
        0xcb => image::Rgb([0x00, 0x66, 0x00]),
        0xcc => image::Rgb([0x00, 0x33, 0xff]),
        0xcd => image::Rgb([0x00, 0x33, 0xcc]),
        0xce => image::Rgb([0x00, 0x33, 0x99]),
        0xcf => image::Rgb([0x00, 0x33, 0x66]),
        0xd0 => image::Rgb([0x00, 0x33, 0x33]),
        0xd1 => image::Rgb([0x00, 0x33, 0x00]),
        0xd2 => image::Rgb([0x00, 0x00, 0xff]),
        0xd3 => image::Rgb([0x00, 0x00, 0xcc]),
        0xd4 => image::Rgb([0x00, 0x00, 0x99]),
        0xd5 => image::Rgb([0x00, 0x00, 0x66]),
        0xd6 => image::Rgb([0x00, 0x00, 0x33]),
        0xd7 => image::Rgb([0xee, 0x00, 0x00]),
        0xd8 => image::Rgb([0xdd, 0x00, 0x00]),
        0xd9 => image::Rgb([0xbb, 0x00, 0x00]),
        0xda => image::Rgb([0xaa, 0x00, 0x00]),
        0xdb => image::Rgb([0x88, 0x00, 0x00]),
        0xdc => image::Rgb([0x77, 0x00, 0x00]),
        0xdd => image::Rgb([0x55, 0x00, 0x00]),
        0xde => image::Rgb([0x44, 0x00, 0x00]),
        0xdf => image::Rgb([0x22, 0x00, 0x00]),
        0xe0 => image::Rgb([0x11, 0x00, 0x00]),
        0xe1 => image::Rgb([0x00, 0xee, 0x00]),
        0xe2 => image::Rgb([0x00, 0xdd, 0x00]),
        0xe3 => image::Rgb([0x00, 0xbb, 0x00]),
        0xe4 => image::Rgb([0x00, 0xaa, 0x00]),
        0xe5 => image::Rgb([0x00, 0x88, 0x00]),
        0xe6 => image::Rgb([0x00, 0x77, 0x00]),
        0xe7 => image::Rgb([0x00, 0x55, 0x00]),
        0xe8 => image::Rgb([0x00, 0x44, 0x00]),
        0xe9 => image::Rgb([0x00, 0x22, 0x00]),
        0xea => image::Rgb([0x00, 0x11, 0x00]),
        0xeb => image::Rgb([0x00, 0x00, 0xee]),
        0xec => image::Rgb([0x00, 0x00, 0xdd]),
        0xed => image::Rgb([0x00, 0x00, 0xbb]),
        0xee => image::Rgb([0x00, 0x00, 0xaa]),
        0xef => image::Rgb([0x00, 0x00, 0x88]),
        0xf0 => image::Rgb([0x00, 0x00, 0x77]),
        0xf1 => image::Rgb([0x00, 0x00, 0x55]),
        0xf2 => image::Rgb([0x00, 0x00, 0x44]),
        0xf3 => image::Rgb([0x00, 0x00, 0x22]),
        0xf4 => image::Rgb([0x00, 0x00, 0x11]),
        0xf5 => image::Rgb([0xee, 0xee, 0xee]),
        0xf6 => image::Rgb([0xdd, 0xdd, 0xdd]),
        0xf7 => image::Rgb([0xbb, 0xbb, 0xbb]),
        0xf8 => image::Rgb([0xaa, 0xaa, 0xaa]),
        0xf9 => image::Rgb([0x88, 0x88, 0x88]),
        0xfa => image::Rgb([0x77, 0x77, 0x77]),
        0xfb => image::Rgb([0x55, 0x55, 0x55]),
        0xfc => image::Rgb([0x44, 0x44, 0x44]),
        0xfd => image::Rgb([0x22, 0x22, 0x22]),
        0xfe => image::Rgb([0x11, 0x11, 0x11]),
        0xff => image::Rgb([0x00, 0x00, 0x00]),
    }
}
