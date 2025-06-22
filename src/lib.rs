
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Size {
    Byte,
    Word,
    Long,
}

impl Size {
    fn decode(d: u16) -> Size {
        match (d >> 6) & 3 {
            0 => Size::Byte,
            1 => Size::Word,
            2 => Size::Long,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SizedImm {
    Byte(u8),
    Word(u16),
    Long(u32),
}

impl SizedImm {
    fn new(size: Size, iter: &mut impl Iterator<Item = u16>) -> Option<SizedImm> {
        match size {
            Size::Byte => Some(SizedImm::Byte((iter.next()?) as u8)),
            Size::Word => Some(SizedImm::Word(iter.next()?)),
            Size::Long => {
                let hi = iter.next()?;
                let lo = iter.next()?;
                let imm = ((hi as u32) << 16) | (lo as u32);
                Some(SizedImm::Long(imm))
            },
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct D(u8);
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct A(u8);

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum IndexReg {
    DReg(D),
    AReg(A),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Addressing {
    DReg(D),
    AReg(A),
    Addr(A),
    AddrPostIncrement(A),
    AddrPreDecrement(A),
    AddrDisplacement(A, u16),
    AddrIndex(u8, A, IndexReg, Size),
    PcDisplacement(u16),
    PcIndex(D, u8),
    AbsoluteShort(u16),
    AbsoluteWord(u32),
    /*ImmediateByte(u8),
    ImmediateWord(u16),
    ImmediateLong(u32),*/
    Immediate,
}

impl Addressing {
    fn decode_mx(m: u8, x: u8, mut v: impl Iterator<Item = u16>) -> Option<Addressing> {
        match m {
            0 => Some(Addressing::DReg(D(x))),
            1 => Some(Addressing::AReg(A(x))),
            2 => Some(Addressing::Addr(A(x))),
            3 => Some(Addressing::AddrPostIncrement(A(x))),
            4 => Some(Addressing::AddrPreDecrement(A(x))),
            5 => Some(Addressing::AddrDisplacement(A(x), v.next()?)),
            6 => {
                let word = v.next()?;
                println!("{:x?}", word);
                let displacement = (word & 0xff) as u8;
                let reg = if ((word & 0x8000) >> 15) == 0 {
                    IndexReg::DReg(D(((word >> 12) & 7) as u8))
                } else {
                    IndexReg::AReg(A(((word >> 12) & 7) as u8))
                };
                let size = if (word >> 1) & 1 == 0 {
                    Size::Word
                } else {
                    Size::Long
                };
                Some(Addressing::AddrIndex(displacement, A(x), reg, size))
            },
            7 => {
                match x {
                    0 => Some(Addressing::AbsoluteShort(v.next()?)),
                    1 => {
                        let hi = v.next()?;
                        let lo = v.next()?;
                        let addr = ((hi as u32) << 16) | (lo as u32);
                        Some(Addressing::AbsoluteWord(addr))
                    },
                    2 => Some(Addressing::PcDisplacement(v.next()?)),
                    3 => Some(Addressing::PcIndex(D(0), 0)),
                    4 => {
                        /*match size {
                            Size::Byte => Some(Addressing::ImmediateByte(v.next()? as u8)),
                            Size::Word => Some(Addressing::ImmediateWord(v.next()?)),
                            Size::Long => {
                                let hi = v.next()?;
                                let lo = v.next()?;
                                let addr = ((hi as u32) << 16) | (lo as u32);
                                Some(Addressing::ImmediateLong(addr))
                            },
                        }*/
                        Some(Addressing::Immediate)
                    },
                    _ => None,
                }
            },
            _ => unreachable!(),
        }
    }
    fn decode(d: u16, v: impl Iterator<Item = u16>) -> Option<Addressing> {
        let m = ((d & 0o70) >> 3) as u8;
        let x = (d & 0o7) as u8;
        Self::decode_mx(m, x, v)
    }
    fn decode_left(d: u16, v: impl Iterator<Item = u16>) -> Option<Addressing> {
        let m = ((d & 0o0700) >> 6) as u8;
        let x = ((d & 0o7000) >> 9) as u8;
        Self::decode_mx(m, x, v)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BitOp {
    Tst,
    Chg,
    Clr,
    Set,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Direction {
    ToRegister,
    ToMemory,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Instruction {
    OriCcr(u8),
    OriSr(u16),
    Ori(SizedImm, Addressing),
    AndiCcr(u8),
    AndiSr(u16),
    Andi(SizedImm, Addressing),
    EoriCcr(u8),
    EoriSr(u16),
    Eori(SizedImm, Addressing),
    Subi(SizedImm, Addressing),
    Addi(SizedImm, Addressing),
    Cmpi(SizedImm, Addressing),
    Bit(BitOp, D, Addressing),
    BitImm(BitOp, u8, Addressing),
    Movep(Size, Direction, D, A, u16),
    Movea(Size, A, Addressing),
    Move(Size, Addressing, Addressing),
}

pub fn decode(v: impl IntoIterator<Item = u16>) -> Option<Instruction> {
    let mut iter = v.into_iter();
    let op = iter.next()?;
    let nibbles: [u8; 4] = [
        (op & 0x000f) as u8,
        ((op & 0x00f0) >> 4) as u8,
        ((op & 0x0f00) >> 8) as u8,
        ((op & 0xf000) >> 12) as u8,
    ];
    let octs: [u8; 4] = [
        (op & 0o0007) as u8,
        ((op & 0o0070) >> 3) as u8,
        ((op & 0o0700) >> 6) as u8,
        ((op & 0o7000) >> 9) as u8,
    ];
    match nibbles[3] {
        0x0 => {
            match nibbles[2] {
                0x0 | 0x2 | 0xa => {
                    let imm = SizedImm::new(Size::decode(op), &mut iter)?;
                    let addr = Addressing::decode(op, &mut iter)?;
                    if addr == Addressing::Immediate {
                        match (imm, nibbles[2]) {
                            (SizedImm::Byte(b), 0x0) => Some(Instruction::OriCcr(b)),
                            (SizedImm::Word(w), 0x0) => Some(Instruction::OriSr(w)),
                            (SizedImm::Byte(b), 0x2) => Some(Instruction::AndiCcr(b)),
                            (SizedImm::Word(w), 0x2) => Some(Instruction::AndiSr(w)),
                            (SizedImm::Byte(b), 0xa) => Some(Instruction::EoriCcr(b)),
                            (SizedImm::Word(w), 0xa) => Some(Instruction::EoriSr(w)),
                            _ => None,
                        }
                    } else {
                        match nibbles[2] {
                            0x0 => Some(Instruction::Ori(imm, addr)),
                            0x2 => Some(Instruction::Andi(imm, addr)),
                            0xa => Some(Instruction::Eori(imm, addr)),
                            _ => unreachable!(),
                        }
                    }
                },
                0x4 => Some(Instruction::Subi(SizedImm::new(Size::decode(op), &mut iter)?, Addressing::decode(op, &mut iter)?)),
                0x6 => Some(Instruction::Addi(SizedImm::new(Size::decode(op), &mut iter)?, Addressing::decode(op, &mut iter)?)),
                0xc => Some(Instruction::Cmpi(SizedImm::new(Size::decode(op), &mut iter)?, Addressing::decode(op, &mut iter)?)),
                _ if octs[1] != 1 => {
                    match octs[2] {
                        0 => Some(Instruction::BitImm(BitOp::Tst, iter.next()? as u8, Addressing::decode(op, &mut iter)?)),
                        1 => Some(Instruction::BitImm(BitOp::Chg, iter.next()? as u8, Addressing::decode(op, &mut iter)?)),
                        2 => Some(Instruction::BitImm(BitOp::Clr, iter.next()? as u8, Addressing::decode(op, &mut iter)?)),
                        3 => Some(Instruction::BitImm(BitOp::Set, iter.next()? as u8, Addressing::decode(op, &mut iter)?)),
                        4 => Some(Instruction::Bit(BitOp::Tst, D(octs[3]), Addressing::decode(op, &mut iter)?)),
                        5 => Some(Instruction::Bit(BitOp::Chg, D(octs[3]), Addressing::decode(op, &mut iter)?)),
                        6 => Some(Instruction::Bit(BitOp::Clr, D(octs[3]), Addressing::decode(op, &mut iter)?)),
                        7 => Some(Instruction::Bit(BitOp::Set, D(octs[3]), Addressing::decode(op, &mut iter)?)),
                        _ => unreachable!(),
                    }
                },
                _ => {
                    let dir = if (octs[2] & 0b010) == 0 {
                        Direction::ToMemory
                    } else {
                        Direction::ToRegister
                    };
                    let size = if (octs[2] & 0b001) == 0 {
                        Size::Word
                    } else {
                        Size::Long
                    };
                    Some(Instruction::Movep(size, dir, D(octs[3]), A(octs[0]), iter.next()?))
                }
            }
        },
        1 | 2 | 3 => {
            let size = match nibbles[3] & 0x3 {
                1 => Size::Byte,
                2 => Size::Long,
                3 => Size::Word,
                _ => unreachable!(),
            };
            let src = Addressing::decode(op, &mut iter)?;
            if octs[2] == 1 && size != Size::Byte {
                Some(Instruction::Movea(size, A(octs[3]), src))
            } else {
                let dst = Addressing::decode_left(op, &mut iter)?;
                Some(Instruction::Move(size, dst, src))
            }
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Instruction, Addressing, Size, D, SizedImm, A, IndexReg, BitOp, Direction};

    // NOTICE
    // Test data shamelessly stolen from
    // https://info.sonicretro.org/SCHG:68000_ASM-to-Hex_Code_Reference

    macro_rules! test {
        ($bytes: expr, $decoded: expr) => {
            assert_eq!(
                super::decode($bytes),
                Some($decoded)
            )
        }
    }

    #[test]
    fn ori() {
        test!(
            [0x0001, 0x0036],
            Instruction::Ori(SizedImm::Byte(0x36), Addressing::DReg(D(1)))
        );
        test!(
            [0x0042, 0x0100],
            Instruction::Ori(SizedImm::Word(0x100), Addressing::DReg(D(2)))
        );
        test!(
            [0x0080, 0x00ff, 0xffff],
            Instruction::Ori(SizedImm::Long(0x00ff_ffff), Addressing::DReg(D(0)))
        );
        test!(
            [0x0038, 0x0070, 0xf100],
            Instruction::Ori(SizedImm::Byte(0x70), Addressing::AbsoluteShort(0xf100))
        );
        test!(
            [0x0078, 0x2005, 0xf100],
            Instruction::Ori(SizedImm::Word(0x2005), Addressing::AbsoluteShort(0xf100))
        );
        test!(
            [0x0011, 0x0080],
            Instruction::Ori(SizedImm::Byte(0x80), Addressing::Addr(A(1)))
        );
        test!(
            [0x0029, 0x0080, 0x002b],
            Instruction::Ori(SizedImm::Byte(0x80), Addressing::AddrDisplacement(A(1), 0x2b))
        );
        test!(
            [0x0019, 0x0080],
            Instruction::Ori(SizedImm::Byte(0x80), Addressing::AddrPostIncrement(A(1)))
        );
        test!(
            [0x0021, 0x0080],
            Instruction::Ori(SizedImm::Byte(0x80), Addressing::AddrPreDecrement(A(1)))
        );
        test!(
            [0x006c, 0x0777, 0x0004],
            Instruction::Ori(SizedImm::Word(0x0777), Addressing::AddrDisplacement(A(4), 4))
        );
        test!(
            [0x0032, 0x007f, 0x0000],
            Instruction::Ori(SizedImm::Byte(0x7f), Addressing::AddrIndex(0, A(2), IndexReg::DReg(D(0)), Size::Word))
        );
        test!(
            [0x0072, 0x07ff, 0x4010],
            Instruction::Ori(SizedImm::Word(0x7ff), Addressing::AddrIndex(0x10, A(2), IndexReg::DReg(D(4)), Size::Word))
        );
        test!(
            [0x007c, 0x0001],
            Instruction::OriSr(1)
        );
    }
    #[test]
    fn andi() {
        test!(
            [0x0201, 0x0036],
            Instruction::Andi(SizedImm::Byte(0x36), Addressing::DReg(D(1)))
        );
        test!(
            [0x0242, 0x0100],
            Instruction::Andi(SizedImm::Word(0x100), Addressing::DReg(D(2)))
        );
        test!(
            [0x0280, 0x00ff, 0xffff],
            Instruction::Andi(SizedImm::Long(0xff_ffff), Addressing::DReg(D(0)))
        );
        test!(
            [0x0238, 0x0070, 0xf100],
            Instruction::Andi(SizedImm::Byte(0x70), Addressing::AbsoluteShort(0xf100))
        );
        test!(
            [0x0278, 0x2005, 0xf100],
            Instruction::Andi(SizedImm::Word(0x2005), Addressing::AbsoluteShort(0xf100))
        );
        test!(
            [0x211, 0x0080],
            Instruction::Andi(SizedImm::Byte(0x80), Addressing::Addr(A(1)))
        );
        test!(
            [0x0229, 0x0080, 0x002b],
            Instruction::Andi(SizedImm::Byte(0x80), Addressing::AddrDisplacement(A(1), 0x2b))
        );
        test!(
            [0x0219, 0x0080],
            Instruction::Andi(SizedImm::Byte(0x80), Addressing::AddrPostIncrement(A(1)))
        );
        test!(
            [0x221, 0x0080],
            Instruction::Andi(SizedImm::Byte(0x80), Addressing::AddrPreDecrement(A(1)))
        );
        test!(
            [0x026c, 0x0777, 0x0004],
            Instruction::Andi(SizedImm::Word(0x0777), Addressing::AddrDisplacement(A(4), 0x4))
        );
        test!(
            [0x0232, 0x007f, 0x0000],
            Instruction::Andi(SizedImm::Byte(0x7f), Addressing::AddrIndex(0x0, A(2), IndexReg::DReg(D(0)), Size::Word))
        );
        test!(
            [0x0272, 0x07ff, 0x4010],
            Instruction::Andi(SizedImm::Word(0x7ff), Addressing::AddrIndex(0x10, A(2), IndexReg::DReg(D(4)), Size::Word))
        );
        test!(
            [0x027c, 0xfffe],
            Instruction::AndiSr(0xfffe)
        );
    }
    #[test]
    fn subi() {
        test!(
            [0x0401, 0x0020],
            Instruction::Subi(SizedImm::Byte(0x20), Addressing::DReg(D(1)))
        );
        test!(
            [0x0480, 0xffff, 0xe6ac],
            Instruction::Subi(SizedImm::Long(0xffffe6ac), Addressing::DReg(D(0)))
        );
        test!(
            [0x478, 0x1337, 0xf100],
            Instruction::Subi(SizedImm::Word(0x1337), Addressing::AbsoluteShort(0xf100))
        );
        test!(
            [0x479, 0x1337, 0xffff, 0xf100],
            Instruction::Subi(SizedImm::Word(0x1337), Addressing::AbsoluteWord(0xffff_f100))
        );
        test!(
            [0x4b8, 0x1965, 0x0917, 0xf100],
            Instruction::Subi(SizedImm::Long(0x1965_0917), Addressing::AbsoluteShort(0xf100))
        );
        test!(
            [0x0411, 0x0040],
            Instruction::Subi(SizedImm::Byte(0x40), Addressing::Addr(A(1)))
        );
        test!(
            [0x469, 0x0040, 0x001c],
            Instruction::Subi(SizedImm::Word(0x40), Addressing::AddrDisplacement(A(1), 0x1c))
        );
        test!(
            [0x4a9, 0x0000, 0x0500, 0x0064],
            Instruction::Subi(SizedImm::Long(0x500), Addressing::AddrDisplacement(A(1), 0x64))
        );
        test!(
            [0x419, 0x0040],
            Instruction::Subi(SizedImm::Byte(0x40), Addressing::AddrPostIncrement(A(1)))
        );
        test!(
            [0x421, 0x0040],
            Instruction::Subi(SizedImm::Byte(0x40), Addressing::AddrPreDecrement(A(1)))
        );
    }
    #[test]
    fn addi() {
        test!(
            [0x0601, 0x0020],
            Instruction::Addi(SizedImm::Byte(0x20), Addressing::DReg(D(1)))
        );
        test!(
            [0x680, 0xffff, 0xe6ac],
            Instruction::Addi(SizedImm::Long(0xffff_e6ac), Addressing::DReg(D(0)))
        );
        test!(
            [0x0678, 0x1337, 0xf100],
            Instruction::Addi(SizedImm::Word(0x1337), Addressing::AbsoluteShort(0xf100))
        );
        test!(
            [0x0679, 0x1337, 0xffff, 0xf100],
            Instruction::Addi(SizedImm::Word(0x1337), Addressing::AbsoluteWord(0xfffff100))
        );
        test!(
            [0x06b8, 0x1965, 0x0917, 0xf100],
            Instruction::Addi(SizedImm::Long(0x1965_0917), Addressing::AbsoluteShort(0xf100))
        );
        test!(
            [0x611, 0x0040],
            Instruction::Addi(SizedImm::Byte(0x40), Addressing::Addr(A(1)))
        );
        test!(
            [0x0669, 0x0040, 0x001c],
            Instruction::Addi(SizedImm::Word(0x40), Addressing::AddrDisplacement(A(1), 0x1c))
        );
        test!(
            [0x06a9, 0x0000, 0x0500, 0x0064],
            Instruction::Addi(SizedImm::Long(0x500), Addressing::AddrDisplacement(A(1), 0x64))
        );
        test!(
            [0x0619, 0x0040],
            Instruction::Addi(SizedImm::Byte(0x40), Addressing::AddrPostIncrement(A(1)))
        );
        test!(
            [0x0621, 0x0040],
            Instruction::Addi(SizedImm::Byte(0x40), Addressing::AddrPreDecrement(A(1)))
        );
    }
    #[test]
    fn eori() {
        test!(
            [0x0a47, 0x8810],
            Instruction::Eori(SizedImm::Word(0x8810), Addressing::DReg(D(7)))
        );
        test!(
            [0x0a38, 0x0080, 0xf00e],
            Instruction::Eori(SizedImm::Byte(0x80), Addressing::AbsoluteShort(0xf00e))
        );
        test!(
            [0x0a78, 0x3119, 0xf010],
            Instruction::Eori(SizedImm::Word(0x3119), Addressing::AbsoluteShort(0xf010))
        );
        test!(
            [0x0a85, 0x1000, 0x1000],
            Instruction::Eori(SizedImm::Long(0x10001000), Addressing::DReg(D(5)))
        );
        test!(
            [0x0a90, 0x0000, 0xffff],
            Instruction::Eori(SizedImm::Long(0xffff), Addressing::Addr(A(0)))
        );
        test!(
            [0x0aa8, 0x0000, 0xffff, 0x0040],
            Instruction::Eori(SizedImm::Long(0xffff), Addressing::AddrDisplacement(A(0), 0x40))
        );
        test!(
            [0xa98, 0x0000, 0xffff],
            Instruction::Eori(SizedImm::Long(0xffff), Addressing::AddrPostIncrement(A(0)))
        );
        test!(
            [0x0aa0, 0x000, 0xffff],
            Instruction::Eori(SizedImm::Long(0xffff), Addressing::AddrPreDecrement(A(0)))
        );
        test!(
            [0x0a7c, 0x0001],
            Instruction::EoriSr(0x0001)
        );
    }
    #[test]
    fn cmpi() {
        test!(
            [0x0c38, 0x0026, 0xffe4],
            Instruction::Cmpi(SizedImm::Byte(0x26), Addressing::AbsoluteShort(0xffe4))
        );
        test!(
            [0x0c78, 0x4170, 0xffb0],
            Instruction::Cmpi(SizedImm::Word(0x4170), Addressing::AbsoluteShort(0xffb0))
        );
        test!(
            [0x0cb8, 0xfffe, 0x8000, 0xee9c],
            Instruction::Cmpi(SizedImm::Long(0xfffe8000), Addressing::AbsoluteShort(0xee9c))
        );
        test!(
            [0x0c79, 0x0003, 0xffff, 0xff08],
            Instruction::Cmpi(SizedImm::Word(0x3), Addressing::AbsoluteWord(0xffff_ff08))
        );
        test!(
            [0x0c00, 0x0002],
            Instruction::Cmpi(SizedImm::Byte(0x2), Addressing::DReg(D(0)))
        );
        test!(
            [0x0c43, 0x02e0],
            Instruction::Cmpi(SizedImm::Word(0x02e0), Addressing::DReg(D(3)))
        );
        test!(
            [0x0c82, 0x0000, 0x0000],
            Instruction::Cmpi(SizedImm::Long(0x0), Addressing::DReg(D(2)))
        );
        test!(
            [0x0c10, 0x0002],
            Instruction::Cmpi(SizedImm::Byte(0x2), Addressing::Addr(A(0)))
        );
        test!(
            [0x0c29, 0x0002, 0x0020],
            Instruction::Cmpi(SizedImm::Byte(0x2), Addressing::AddrDisplacement(A(1), 0x20))
        );
        test!(
            [0x0c68, 0x1044, 0x0010],
            Instruction::Cmpi(SizedImm::Word(0x1044), Addressing::AddrDisplacement(A(0), 0x10))
        );
        test!(
            [0x0c18, 0x0002],
            Instruction::Cmpi(SizedImm::Byte(0x2), Addressing::AddrPostIncrement(A(0)))
        );
        test!(
            [0x0c20, 0x0002],
            Instruction::Cmpi(SizedImm::Byte(0x2), Addressing::AddrPreDecrement(A(0)))
        );
    }
    #[test]
    fn btst() {
        test!(
            [0x0838, 0x0006, 0xf604],
            Instruction::BitImm(BitOp::Tst, 6, Addressing::AbsoluteShort(0xf604))
        );
        test!(
            [0x801, 0x0004],
            Instruction::BitImm(BitOp::Tst, 4, Addressing::DReg(D(1)))
        );
        test!(
            [0x0810, 0x0006],
            Instruction::BitImm(BitOp::Tst, 6, Addressing::Addr(A(0)))
        );
        test!(
            [0x0828, 0x0006, 0x002a],
            Instruction::BitImm(BitOp::Tst, 6, Addressing::AddrDisplacement(A(0), 0x2a))
        );
        test!(
            [0x0d38, 0xf604],
            Instruction::Bit(BitOp::Tst, D(6), Addressing::AbsoluteShort(0xf604))
        );
        test!(
            [0x0d01],
            Instruction::Bit(BitOp::Tst, D(6), Addressing::DReg(D(1)))
        );
        test!(
            [0x0d10],
            Instruction::Bit(BitOp::Tst, D(6), Addressing::Addr(A(0)))
        );
        test!(
            [0x0d28, 0x002a],
            Instruction::Bit(BitOp::Tst, D(6), Addressing::AddrDisplacement(A(0), 0x2a))
        );
    }
    #[test]
    fn bclr() {
        test!(
            [0x08b8, 0x0007, 0xe43d],
            Instruction::BitImm(BitOp::Clr, 7, Addressing::AbsoluteShort(0xe43d))
        );
        test!(
            [0x0882, 0x0007],
            Instruction::BitImm(BitOp::Clr, 7, Addressing::DReg(D(2)))
        );
        test!(
            [0x891, 0x0003],
            Instruction::BitImm(BitOp::Clr, 3, Addressing::Addr(A(1)))
        );
        test!(
            [0x08a9, 0x0003, 0x002a],
            Instruction::BitImm(BitOp::Clr, 3, Addressing::AddrDisplacement(A(1), 0x2a))
        );
        test!(
            [0x0db8, 0xe43d],
            Instruction::Bit(BitOp::Clr, D(6), Addressing::AbsoluteShort(0xe43d))
        );
        test!(
            [0x0d82],
            Instruction::Bit(BitOp::Clr, D(6), Addressing::DReg(D(2)))
        );
        test!(
            [0x0d90],
            Instruction::Bit(BitOp::Clr, D(6), Addressing::Addr(A(0)))
        );
        test!(
            [0x0da8, 0x002a],
            Instruction::Bit(BitOp::Clr, D(6), Addressing::AddrDisplacement(A(0), 0x2a))
        );
    }
    #[test]
    fn bset() {
        test!(
            [0x08f8, 0x0003, 0xfe05],
            Instruction::BitImm(BitOp::Set, 3, Addressing::AbsoluteShort(0xfe05))
        );
        test!(
            [0x08c2, 0x0003],
            Instruction::BitImm(BitOp::Set, 3, Addressing::DReg(D(2)))
        );
        test!(
            [0x08d0, 0x0002],
            Instruction::BitImm(BitOp::Set, 2, Addressing::Addr(A(0)))
        );
        test!(
            [0x08e8, 0x0002, 0x002a],
            Instruction::BitImm(BitOp::Set, 2, Addressing::AddrDisplacement(A(0), 0x2a))
        );
        test!(
            [0x01f8, 0xfe05],
            Instruction::Bit(BitOp::Set, D(0), Addressing::AbsoluteShort(0xfe05))
        );
        test!(
            [0x01c2],
            Instruction::Bit(BitOp::Set, D(0), Addressing::DReg(D(2)))
        );
        test!(
            [0x01d2],
            Instruction::Bit(BitOp::Set, D(0), Addressing::Addr(A(2)))
        );
        test!(
            [0x01ea, 0x002a],
            Instruction::Bit(BitOp::Set, D(0), Addressing::AddrDisplacement(A(2), 0x2a))
        );
    }
    #[test]
    fn bchg() {
        test!(
            [0x0878, 0x0004, 0xffb2],
            Instruction::BitImm(BitOp::Chg, 4, Addressing::AbsoluteShort(0xffb2))
        );
        test!(
            [0x0842, 0x0004],
            Instruction::BitImm(BitOp::Chg, 4, Addressing::DReg(D(2)))
        );
        test!(
            [0x0853, 0x0004],
            Instruction::BitImm(BitOp::Chg, 4, Addressing::Addr(A(3)))
        );
        test!(
            [0x086b, 0x0004, 0x002a],
            Instruction::BitImm(BitOp::Chg, 4, Addressing::AddrDisplacement(A(3), 0x2a))
        );
        test!(
            [0x0778, 0xffb2],
            Instruction::Bit(BitOp::Chg, D(3), Addressing::AbsoluteShort(0xffb2))
        );
        test!(
            [0x0742],
            Instruction::Bit(BitOp::Chg, D(3), Addressing::DReg(D(2)))
        );
        test!(
            [0x0753],
            Instruction::Bit(BitOp::Chg, D(3), Addressing::Addr(A(3)))
        );
        test!(
            [0x076b, 0x002a],
            Instruction::Bit(BitOp::Chg, D(3), Addressing::AddrDisplacement(A(3), 0x2a))
        );
    }
    #[test]
    fn movep() {
        test!(
            [0x0708, 0x0000],
            Instruction::Movep(Size::Word, Direction::ToMemory, D(3), A(0), 0)
        );
        test!(
            [0x0748, 0x0000],
            Instruction::Movep(Size::Long, Direction::ToMemory, D(3), A(0), 0)
        );
        test!(
            [0x0588, 0x0000],
            Instruction::Movep(Size::Word, Direction::ToRegister, D(2), A(0), 0)
        );
        test!(
            [0x05c8, 0x0000],
            Instruction::Movep(Size::Long, Direction::ToRegister, D(2), A(0), 0)
        );
    }
/*    #[test]
    fn r#move() {
        test!(
            [0x11fc, 0x0064, 0xffe0],
            Instruction::Move(Addressing::ImmediateByte(0x64))
        );
    }*/
}
