
pub trait Fetch {
    fn next_u8(&mut self) -> Option<u8>;
    fn next_u16(&mut self) -> Option<u16>;
    fn next_u32(&mut self) -> Option<u32>;
}

impl<T: Iterator<Item = u16>> Fetch for T {
    fn next_u8(&mut self) -> Option<u8> {
        self.next().map(|v| v as u8)
    }
    fn next_u16(&mut self) -> Option<u16> {
        self.next()
    }
    fn next_u32(&mut self) -> Option<u32> {
        let hi = self.next()?;
        let lo = self.next()?;
        Some(((hi as u32) << 16) | (lo as u32))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Size {
    Byte,
    Word,
    Long,
}

impl Size {
    fn decode(d: u16) -> Option<Size> {
        match (d >> 6) & 3 {
            0 => Some(Size::Byte),
            1 => Some(Size::Word),
            2 => Some(Size::Long),
            _ => None,
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
    fn new(size: Size, iter: &mut impl Fetch) -> Option<SizedImm> {
        match size {
            Size::Byte => Some(SizedImm::Byte(iter.next_u8()?)),
            Size::Word => Some(SizedImm::Word(iter.next_u16()?)),
            Size::Long => Some(SizedImm::Long(iter.next_u32()?)),
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
pub enum Rm {
    R(D, D),
    M(A, A),
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
    PcIndex(u8, IndexReg, Size),
    AbsoluteShort(u16),
    AbsoluteWord(u32),
    ImmediateByte(u8),
    ImmediateWord(u16),
    ImmediateLong(u32),
}

impl Addressing {
    fn decode_mx(m: u8, x: u8, size: Option<Size>, v: &mut impl Fetch) -> Option<Addressing> {
        match m {
            0 => Some(Addressing::DReg(D(x))),
            1 => Some(Addressing::AReg(A(x))),
            2 => Some(Addressing::Addr(A(x))),
            3 => Some(Addressing::AddrPostIncrement(A(x))),
            4 => Some(Addressing::AddrPreDecrement(A(x))),
            5 => Some(Addressing::AddrDisplacement(A(x), v.next_u16()?)),
            6 => {
                let word = v.next_u16()?;
                let displacement = (word & 0xff) as u8;
                let reg = if ((word & 0x8000) >> 15) == 0 {
                    IndexReg::DReg(D(((word >> 12) & 7) as u8))
                } else {
                    IndexReg::AReg(A(((word >> 12) & 7) as u8))
                };
                let size = if word & 0x800 == 0 {
                    Size::Word
                } else {
                    Size::Long
                };
                Some(Addressing::AddrIndex(displacement, A(x), reg, size))
            },
            7 => {
                match x {
                    0 => Some(Addressing::AbsoluteShort(v.next_u16()?)),
                    1 => Some(Addressing::AbsoluteWord(v.next_u32()?)),
                    2 => Some(Addressing::PcDisplacement(v.next_u16()?)),
                    3 => {
                        let word = v.next_u16()?;
                        let displacement = (word & 0xff) as u8;
                        let reg = if ((word & 0x8000) >> 15) == 0 {
                            IndexReg::DReg(D(((word >> 12) & 7) as u8))
                        } else {
                            IndexReg::AReg(A(((word >> 12) & 7) as u8))
                        };
                        let size = if word & 0x800 == 0 {
                            Size::Word
                        } else {
                            Size::Long
                        };
                        Some(Addressing::PcIndex(displacement, reg, size))
                    },
                    4 => {
                        match size? {
                            Size::Byte => Some(Addressing::ImmediateByte(v.next_u8()?)),
                            Size::Word => Some(Addressing::ImmediateWord(v.next_u16()?)),
                            Size::Long => Some(Addressing::ImmediateLong(v.next_u32()?)),
                        }
                    },
                    _ => None,
                }
            },
            _ => unreachable!(),
        }
    }
    fn noimm(d: u16, v: &mut impl Fetch) -> Option<Addressing> {
        Self::decode(d, None, v)
    }
    fn decode(d: u16, size: Option<Size>, v: &mut impl Fetch) -> Option<Addressing> {
        let m = ((d & 0o70) >> 3) as u8;
        let x = (d & 0o7) as u8;
        Self::decode_mx(m, x, size, v)
    }
    fn decode_left(d: u16, size: Option<Size>, v: &mut impl Fetch) -> Option<Addressing> {
        let m = ((d & 0o0700) >> 6) as u8;
        let x = ((d & 0o7000) >> 9) as u8;
        Self::decode_mx(m, x, size, v)
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
pub enum OpResult {
    Register,
    EffectiveAddress,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Condition {
    True,
    False,
    Higher,
    LowerOrSame,
    CarryClear,
    CarrySet,
    NotEqual,
    Equal,
    OverflowClear,
    OverflowSet,
    Plus,
    Minus,
    GreaterOrEqual,
    LessThan,
    GreaterThan,
    LessOrEqual,
}

impl Condition {
    fn decode(v: u16) -> Condition {
        match (v & 0x0f00) >> 8 {
            0x0 => Condition::True,
            0x1 => Condition::False,
            0x2 => Condition::Higher,
            0x3 => Condition::LowerOrSame,
            0x4 => Condition::CarryClear,
            0x5 => Condition::CarrySet,
            0x6 => Condition::NotEqual,
            0x7 => Condition::Equal,
            0x8 => Condition::OverflowClear,
            0x9 => Condition::OverflowSet,
            0xa => Condition::Plus,
            0xb => Condition::Minus,
            0xc => Condition::GreaterOrEqual,
            0xd => Condition::LessThan,
            0xe => Condition::GreaterThan,
            0xf => Condition::LessOrEqual,
            _ => unreachable!(),
        }
    }
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
    Movea(Size, Addressing, A),
    Move(Size, Addressing, Addressing),
    // barely tested
    MoveFromSr(Addressing),
    // untested
    MoveToCcr(Addressing),
    // untested
    MoveToSr(Addressing),
    // untested
    Negx(Size, Addressing),
    Clr(Size, Addressing),
    Neg(Size, Addressing),
    Not(Size, Addressing),
    // untested
    Nbcd(Addressing),
    Swap(D),
    Ext(Size, D),
    Pea(Addressing),
    Illegal,
    Tas(Addressing),
    Tst(Size, Addressing),
    Trap(u8),
    Link(A, u16),
    Unlk(A),
    // untested
    MoveUsp(Direction, A),
    // untested
    Reset,
    Nop,
    Stop(u16),
    Rte,
    Rts,
    Trapv,
    Rtr,
    Jsr(Addressing),
    Jmp(Addressing),
    // untested
    Movem(Direction, Size, Addressing, u16),
    Lea(Addressing, A),
    // untested
    Chk(D, Addressing),
    Addq(Size, u8, Addressing),
    Subq(Size, u8, Addressing),
    S(Condition, Addressing),
    Db(Condition, D, u16),
    Bra(u16),
    Bsr(u16),
    B(Condition, u16),
    Moveq(u8, D),
    // untested
    Sbcd(Rm),
    Divu(Addressing, D),
    Divs(Addressing, D),
    ATrap(u16),
}

pub struct Decoder<T: Iterator<Item = u16>>(T);

impl<T: Iterator<Item = u16>> Iterator for Decoder<T> {
    type Item = Instruction;
    fn next(&mut self) -> Option<Instruction> {
        decode(&mut self.0)
    }
}

pub fn decode_iter<T: IntoIterator<Item = u16>>(iter: T) -> Decoder<<T as IntoIterator>::IntoIter> {
    Decoder(iter.into_iter())
}

pub fn decode(iter: impl IntoIterator<Item = u16>) -> Option<Instruction> {
    decode_inner(&mut iter.into_iter())
}

pub fn decode_inner(iter: &mut impl Fetch) -> Option<Instruction> {
    let op = iter.next_u16()?;
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
        0xa => Some(Instruction::ATrap(op)),
        0x0 => {
            match nibbles[2] {
                0x0 | 0x2 | 0xa => {
                    let size = SizedImm::new(Size::decode(op)?, iter)?;
                    if let Some(addr) = Addressing::decode(op, None, iter) {
                        match octs[3] {
                            0 => Some(Instruction::Ori(size, addr)),
                            1 => Some(Instruction::Andi(size, addr)),
                            5 => Some(Instruction::Eori(size, addr)),
                            _ => None,
                        }
                    } else {
                        match (octs[3], size) {
                            (0, SizedImm::Byte(b)) => Some(Instruction::OriCcr(b)),
                            (0, SizedImm::Word(w)) => Some(Instruction::OriSr(w)),
                            (1, SizedImm::Byte(b)) => Some(Instruction::AndiCcr(b)),
                            (1, SizedImm::Word(w)) => Some(Instruction::AndiSr(w)),
                            (5, SizedImm::Byte(b)) => Some(Instruction::EoriCcr(b)),
                            (5, SizedImm::Word(w)) => Some(Instruction::EoriSr(w)),
                            _ => None
                        }
                    }
                },
                0x4 => Some(Instruction::Subi(SizedImm::new(Size::decode(op)?, iter)?, Addressing::noimm(op, iter)?)),
                0x6 => Some(Instruction::Addi(SizedImm::new(Size::decode(op)?, iter)?, Addressing::noimm(op, iter)?)),
                0xc => Some(Instruction::Cmpi(SizedImm::new(Size::decode(op)?, iter)?, Addressing::noimm(op, iter)?)),
                _ if octs[1] != 1 => {
                    match octs[2] {
                        0 => Some(Instruction::BitImm(BitOp::Tst, iter.next_u8()?, Addressing::noimm(op, iter)?)),
                        1 => Some(Instruction::BitImm(BitOp::Chg, iter.next_u8()?, Addressing::noimm(op, iter)?)),
                        2 => Some(Instruction::BitImm(BitOp::Clr, iter.next_u8()?, Addressing::noimm(op, iter)?)),
                        3 => Some(Instruction::BitImm(BitOp::Set, iter.next_u8()?, Addressing::noimm(op, iter)?)),
                        4 => Some(Instruction::Bit(BitOp::Tst, D(octs[3]), Addressing::noimm(op, iter)?)),
                        5 => Some(Instruction::Bit(BitOp::Chg, D(octs[3]), Addressing::noimm(op, iter)?)),
                        6 => Some(Instruction::Bit(BitOp::Clr, D(octs[3]), Addressing::noimm(op, iter)?)),
                        7 => Some(Instruction::Bit(BitOp::Set, D(octs[3]), Addressing::noimm(op, iter)?)),
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
                    Some(Instruction::Movep(size, dir, D(octs[3]), A(octs[0]), iter.next_u16()?))
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
            let src = Addressing::decode(op, Some(size), iter)?;
            if octs[2] == 1 && size != Size::Byte {
                Some(Instruction::Movea(size, src, A(octs[3])))
            } else {
                let dst = Addressing::decode_left(op, None, iter)?;
                Some(Instruction::Move(size, src, dst))
            }
        },
        4 => {
            match nibbles[2] {
                0b0000 if octs[2] == 3 => Some(Instruction::MoveFromSr(Addressing::noimm(op, iter)?)),
                0b0100 if octs[2] == 3 => Some(Instruction::MoveToCcr(Addressing::decode(op, Some(Size::Byte), iter)?)),
                0b0110 if octs[2] == 3 => Some(Instruction::MoveToSr(Addressing::decode(op, Some(Size::Word), iter)?)),
                0b0000 => Some(Instruction::Negx(Size::decode(op)?, Addressing::noimm(op, iter)?)),
                0b0010 => Some(Instruction::Clr(Size::decode(op)?, Addressing::noimm(op, iter)?)),
                0b0100 => Some(Instruction::Neg(Size::decode(op)?, Addressing::noimm(op, iter)?)),
                0b0110 => Some(Instruction::Not(Size::decode(op)?, Addressing::noimm(op, iter)?)),
                0b1000 => {
                    if octs[1] == 0 {
                        match octs[2] {
                            0 => Some(Instruction::Nbcd(Addressing::noimm(op, iter)?)),
                            1 => Some(Instruction::Swap(D(octs[0]))),
                            2 => Some(Instruction::Ext(Size::Word, D(octs[0]))),
                            3 => Some(Instruction::Ext(Size::Long, D(octs[0]))),
                            _ => unreachable!(),
                        }
                    } else {
                        match octs[2] {
                            0 => Some(Instruction::Nbcd(Addressing::noimm(op, iter)?)),
                            1 => Some(Instruction::Pea(Addressing::decode(op, Some(Size::Long), iter)?)),
                            2 | 3 => None,
                            _ => unreachable!(),
                        }
                    }
                },
                0b1010 => {
                    match Size::decode(op) {
                        Some(size) => Some(Instruction::Tst(size, Addressing::noimm(op, iter)?)),
                        None => {
                            if octs[1] == 0b111 && octs[0] == 0b100 {
                                Some(Instruction::Illegal)
                            } else {
                                Some(Instruction::Tas(Addressing::noimm(op, iter)?))
                            }
                        },
                    }
                },
                0b1110 => {
                    match nibbles[1] {
                        0b0100 => Some(Instruction::Trap(nibbles[0])),
                        0b0101 => {
                            if nibbles[0] & 0x8 == 0 {
                                Some(Instruction::Link(A(octs[0]), iter.next_u16()?))
                            } else {
                                Some(Instruction::Unlk(A(octs[0])))
                            }
                        },
                        0b0110 => {
                            let dir = if nibbles[0] & 0x80 == 0 {
                                Direction::ToMemory
                            } else {
                                Direction::ToRegister
                            };
                            Some(Instruction::MoveUsp(dir, A(octs[0])))
                        },
                        0b0111 => {
                            match nibbles[0] {
                                0b0000 => Some(Instruction::Reset),
                                0b0001 => Some(Instruction::Nop),
                                0b0010 => Some(Instruction::Stop(iter.next_u16()?)),
                                0b0011 => Some(Instruction::Rte),
                                0b0101 => Some(Instruction::Rts),
                                0b0110 => Some(Instruction::Trapv),
                                0b0111 => Some(Instruction::Rtr),
                                _ => None,
                            }
                        },
                        _ => {
                            match octs[2] {
                                0b010 => Some(Instruction::Jsr(Addressing::noimm(op, iter)?)),
                                0b011 => Some(Instruction::Jmp(Addressing::noimm(op, iter)?)),
                                _ => None,
                            }
                        },
                    }
                },
                _ => {
                    match octs[2] {
                        0b011 | 0b010 => {
                            let dir = match octs[3] {
                                0b100 => Direction::ToRegister,
                                0b110 => Direction::ToMemory,
                                _ => None?,
                            };
                            let size = match octs[2] {
                                0b010 => Size::Word,
                                0b011 => Size::Long,
                                _ => unreachable!(),
                            };
                            let mask = iter.next_u16()?;
                            Some(Instruction::Movem(dir, size, Addressing::noimm(op, iter)?, mask))
                        },
                        0b111 => Some(Instruction::Lea(Addressing::noimm(op, iter)?, A(octs[3]))),
                        0b110 => Some(Instruction::Chk(D(octs[3]), Addressing::noimm(op, iter)?)),
                        _ => None,
                    }
                },
            }
        },
        0b0101 => {
            match Size::decode(op) {
                Some(size) => {
                    if octs[2] & 0b100 == 0 {
                        Some(Instruction::Addq(size, octs[3], Addressing::noimm(op, iter)?))
                    } else {
                        Some(Instruction::Subq(size, octs[3], Addressing::noimm(op, iter)?))
                    }
                },
                None => {
                    if octs[1] == 0b001 {
                        Some(Instruction::Db(Condition::decode(op), D(octs[0]), iter.next_u16()?))
                    } else {
                        Some(Instruction::S(Condition::decode(op), Addressing::noimm(op, iter)?))
                    }
                },
            }
        },
        0b0110 => {
            let target = if op & 0xff == 0 {
                iter.next_u16()?
            } else {
                op & 0xff
            };
            match Condition::decode(op) {
                Condition::True => Some(Instruction::Bra(target)),
                Condition::False => Some(Instruction::Bsr(target)),
                cond => Some(Instruction::B(cond, target)),
            }
        },
        0b0111 if nibbles[2] & 1 == 0 => Some(Instruction::Moveq(op as u8, D(octs[3]))),
        0b1000 => {
            match Size::decode(op) {
                Some(_size) => {
                    if nibbles[1] == 0 && nibbles[2] & 1 == 1 {
                        let rm = if octs[1] & 1 == 0 {
                            Rm::R(D(octs[0]), D(octs[3]))
                        } else {
                            Rm::M(A(octs[0]), A(octs[3]))
                        };
                        Some(Instruction::Sbcd(rm))
                    } else {
                        let _dir = if octs[2] & 0b100 == 0 {
                            OpResult::Register
                        } else {
                            OpResult::EffectiveAddress
                        };
                        todo!("or")
                    }
                }
                None => {
                    if octs[2] == 0b011 {
                        Some(Instruction::Divu(Addressing::decode(op, Some(Size::Word), iter)?, D(octs[3])))
                    } else {
                        Some(Instruction::Divs(Addressing::decode(op, Some(Size::Word), iter)?, D(octs[3])))
                    }
                },
            }
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Instruction, Addressing, Size, D, SizedImm, A, IndexReg, BitOp, Direction, Condition};

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
    #[test]
    fn r#move() {
        test!(
            [0x11fc, 0x0064, 0xffe0],
            Instruction::Move(Size::Byte, Addressing::ImmediateByte(0x64), Addressing::AbsoluteShort(0xffe0))
        );
        test!(
            [0x31fc, 0x03e8, 0xffe0],
            Instruction::Move(Size::Word, Addressing::ImmediateWord(0x03e8), Addressing::AbsoluteShort(0xffe0))
        );
        test!(
            [0x21fc, 0x05f5, 0xe100, 0xffe0],
            Instruction::Move(Size::Long, Addressing::ImmediateLong(0x05f5_e100), Addressing::AbsoluteShort(0xffe0))
        );
        test!(
            [0x123c, 0x0020],
            Instruction::Move(Size::Byte, Addressing::ImmediateByte(0x20), Addressing::DReg(D(1)))
        );
        test!(
            [0x203c, 0x6000, 0x0003],
            Instruction::Move(Size::Long, Addressing::ImmediateLong(0x6000_0003), Addressing::DReg(D(0)))
        );
        test!(
            [0x32bc, 0x0101],
            Instruction::Move(Size::Word, Addressing::ImmediateWord(0x101), Addressing::Addr(A(1)))
        );
        test!(
            [0x157c, 0x003c, 0x0002],
            Instruction::Move(Size::Byte, Addressing::ImmediateByte(0x3c), Addressing::AddrDisplacement(A(2), 0x2))
        );
        test!(
            [0x337c, 0x2fa0, 0x0010],
            Instruction::Move(Size::Word, Addressing::ImmediateWord(0x2fa0), Addressing::AddrDisplacement(A(1), 0x10))
        );
        test!(
            [0x36fc, 0x0000],
            Instruction::Move(Size::Word, Addressing::ImmediateWord(0), Addressing::AddrPostIncrement(A(3)))
        );
        test!(
            [0x373c, 0x0000],
            Instruction::Move(Size::Word, Addressing::ImmediateWord(0), Addressing::AddrPreDecrement(A(3)))
        );
        test!(
            [0x31f8, 0xff0a, 0xff08],
            Instruction::Move(Size::Word, Addressing::AbsoluteShort(0xff0a), Addressing::AbsoluteShort(0xff08))
        );
        test!(
            [0x33f8, 0xff0a, 0xffff, 0xff08],
            Instruction::Move(Size::Word, Addressing::AbsoluteShort(0xff0a), Addressing::AbsoluteWord(0xffff_ff08))
        );
        test!(
            [0x31f9, 0xffff, 0x01a5, 0xffe0],
            Instruction::Move(Size::Word, Addressing::AbsoluteWord(0xffff_01a5), Addressing::AbsoluteShort(0xffe0))
        );
        test!(
            [0x3038, 0xee18],
            Instruction::Move(Size::Word, Addressing::AbsoluteShort(0xee18), Addressing::DReg(D(0)))
        );
        test!(
            [0x10b8, 0xff0b],
            Instruction::Move(Size::Byte, Addressing::AbsoluteShort(0xff0b), Addressing::Addr(A(0)))
        );
        test!(
            [0x1178, 0xff0b, 0x0022],
            Instruction::Move(Size::Byte, Addressing::AbsoluteShort(0xff0b), Addressing::AddrDisplacement(A(0), 0x22))
        );
        test!(
            [0x10f8, 0xff0b],
            Instruction::Move(Size::Byte, Addressing::AbsoluteShort(0xff0b), Addressing::AddrPostIncrement(A(0)))
        );
        test!(
            [0x1138, 0xff0b],
            Instruction::Move(Size::Byte, Addressing::AbsoluteShort(0xff0b), Addressing::AddrPreDecrement(A(0)))
        );
        test!(
            [0x11c0, 0xf604],
            Instruction::Move(Size::Byte, Addressing::DReg(D(0)), Addressing::AbsoluteShort(0xf604))
        );
        test!(
            [0x23c1, 0xffff, 0xffe0],
            Instruction::Move(Size::Long, Addressing::DReg(D(1)), Addressing::AbsoluteWord(0xffff_ffe0))
        );
        test!(
            [0x3601],
            Instruction::Move(Size::Word, Addressing::DReg(D(1)), Addressing::DReg(D(3)))
        );
        test!(
            [0x2e00],
            Instruction::Move(Size::Long, Addressing::DReg(D(0)), Addressing::DReg(D(7)))
        );
        test!(
            [0x3c80],
            Instruction::Move(Size::Word, Addressing::DReg(D(0)), Addressing::Addr(A(6)))
        );
        test!(
            [0x1143, 0x0026],
            Instruction::Move(Size::Byte, Addressing::DReg(D(3)), Addressing::AddrDisplacement(A(0), 0x26))
        );
        test!(
            [0x34c3],
            Instruction::Move(Size::Word, Addressing::DReg(D(3)), Addressing::AddrPostIncrement(A(2)))
        );
        test!(
            [0x3503],
            Instruction::Move(Size::Word, Addressing::DReg(D(3)), Addressing::AddrPreDecrement(A(2)))
        );
        test!(
            [0x1213],
            Instruction::Move(Size::Byte, Addressing::Addr(A(3)), Addressing::DReg(D(1)))
        );
        test!(
            [0x1893],
            Instruction::Move(Size::Byte, Addressing::Addr(A(3)), Addressing::Addr(A(4)))
        );
        test!(
            [0x1551, 0x0003],
            Instruction::Move(Size::Byte, Addressing::Addr(A(1)), Addressing::AddrDisplacement(A(2), 3))
        );
        test!(
            [0x2550, 0x0080],
            Instruction::Move(Size::Long, Addressing::Addr(A(0)), Addressing::AddrDisplacement(A(2), 0x80))
        );
        test!(
            [0x18d3],
            Instruction::Move(Size::Byte, Addressing::Addr(A(3)), Addressing::AddrPostIncrement(A(4)))
        );
        test!(
            [0x1913],
            Instruction::Move(Size::Byte, Addressing::Addr(A(3)), Addressing::AddrPreDecrement(A(4)))
        );
        test!(
            [0x31e8, 0x0034, 0xff08],
            Instruction::Move(Size::Word, Addressing::AddrDisplacement(A(0), 0x34), Addressing::AbsoluteShort(0xff08))
        );
        test!(
            [0x1029, 0x0008],
            Instruction::Move(Size::Byte, Addressing::AddrDisplacement(A(1), 0x8), Addressing::DReg(D(0)))
        );
        test!(
            [0x1c28, 0x0026],
            Instruction::Move(Size::Byte, Addressing::AddrDisplacement(A(0), 0x26), Addressing::DReg(D(6)))
        );
        test!(
            [0x2629, 0x0064],
            Instruction::Move(Size::Long, Addressing::AddrDisplacement(A(1), 0x64), Addressing::DReg(D(3)))
        );
        test!(
            [0x14aa, 0x0001],
            Instruction::Move(Size::Byte, Addressing::AddrDisplacement(A(2), 0x1), Addressing::Addr(A(2)))
        );
        test!(
            [0x156a, 0x0003, 0x0002],
            Instruction::Move(Size::Byte, Addressing::AddrDisplacement(A(2), 0x3), Addressing::AddrDisplacement(A(2), 0x2))
        );
        test!(
            [0x3569, 0x0014, 0x0054],
            Instruction::Move(Size::Word, Addressing::AddrDisplacement(A(1), 0x14), Addressing::AddrDisplacement(A(2), 0x54))
        );
        test!(
            [0x2569, 0x0010, 0x0050],
            Instruction::Move(Size::Long, Addressing::AddrDisplacement(A(1), 0x10), Addressing::AddrDisplacement(A(2), 0x50))
        );
        test!(
            [0x14ea, 0x0001],
            Instruction::Move(Size::Byte, Addressing::AddrDisplacement(A(2), 0x1), Addressing::AddrPostIncrement(A(2)))
        );
        test!(
            [0x152a, 0x0001],
            Instruction::Move(Size::Byte, Addressing::AddrDisplacement(A(2), 0x1), Addressing::AddrPreDecrement(A(2)))
        );
        test!(
            [0x11dc, 0xffb2],
            Instruction::Move(Size::Byte, Addressing::AddrPostIncrement(A(4)), Addressing::AbsoluteShort(0xffb2))
        );
        test!(
            [0x101c],
            Instruction::Move(Size::Byte, Addressing::AddrPostIncrement(A(4)), Addressing::DReg(D(0)))
        );
        test!(
            [0x201c],
            Instruction::Move(Size::Long, Addressing::AddrPostIncrement(A(4)), Addressing::DReg(D(0)))
        );
        test!(
            [0x2298],
            Instruction::Move(Size::Long, Addressing::AddrPostIncrement(A(0)), Addressing::Addr(A(1)))
        );
        test!(
            [0x2559, 0x0074],
            Instruction::Move(Size::Long, Addressing::AddrPostIncrement(A(1)), Addressing::AddrDisplacement(A(2), 0x74))
        );
        test!(
            [0x22d8],
            Instruction::Move(Size::Long, Addressing::AddrPostIncrement(A(0)), Addressing::AddrPostIncrement(A(1)))
        );
        test!(
            [0x1023],
            Instruction::Move(Size::Byte, Addressing::AddrPreDecrement(A(3)), Addressing::DReg(D(0)))
        );
        test!(
            [0x3024],
            Instruction::Move(Size::Word, Addressing::AddrPreDecrement(A(4)), Addressing::DReg(D(0)))
        );
        test!(
            [0x3325],
            Instruction::Move(Size::Word, Addressing::AddrPreDecrement(A(5)), Addressing::AddrPreDecrement(A(1)))
        );
        test!(
            [0x21c9, 0xe446],
            Instruction::Move(Size::Long, Addressing::AReg(A(1)), Addressing::AbsoluteShort(0xe446))
        );
        test!(
            [0x2009],
            Instruction::Move(Size::Long, Addressing::AReg(A(1)), Addressing::DReg(D(0)))
        );
        test!(
            [0x2089],
            Instruction::Move(Size::Long, Addressing::AReg(A(1)), Addressing::Addr(A(0)))
        );
        test!(
            [0x314a, 0x003c],
            Instruction::Move(Size::Word, Addressing::AReg(A(2)), Addressing::AddrDisplacement(A(0), 0x3c))
        );
        test!(
            [0x2e8a],
            Instruction::Move(Size::Long, Addressing::AReg(A(2)), Addressing::Addr(A(7)))
        );
        test!(
            [0x2eca],
            Instruction::Move(Size::Long, Addressing::AReg(A(2)), Addressing::AddrPostIncrement(A(7)))
        );
        test!(
            [0x2f0a],
            Instruction::Move(Size::Long, Addressing::AReg(A(2)), Addressing::AddrPreDecrement(A(7)))
        );
        test!(
            [0x11f2, 0x6014, 0xe446],
            Instruction::Move(Size::Byte, Addressing::AddrIndex(0x14, A(2), IndexReg::DReg(D(6)), Size::Word), Addressing::AbsoluteShort(0xe446))
        );
        test!(
            [0x1232, 0x0000],
            Instruction::Move(Size::Byte, Addressing::AddrIndex(0, A(2), IndexReg::DReg(D(0)), Size::Word), Addressing::DReg(D(1)))
        );
        test!(
            [0x3c34, 0x0004],
            Instruction::Move(Size::Word, Addressing::AddrIndex(0x4, A(4), IndexReg::DReg(D(0)), Size::Word), Addressing::DReg(D(6)))
        );
        test!(
            [0x1833, 0x401d],
            Instruction::Move(Size::Byte, Addressing::AddrIndex(0x1d, A(3), IndexReg::DReg(D(4)), Size::Word), Addressing::DReg(D(4)))
        );
        test!(
            [0x16b2, 0x2000],
            Instruction::Move(Size::Byte, Addressing::AddrIndex(0, A(2), IndexReg::DReg(D(2)), Size::Word), Addressing::Addr(A(3)))
        );
        test!(
            [0x1772, 0x6014, 0x003c],
            Instruction::Move(Size::Byte, Addressing::AddrIndex(0x14, A(2), IndexReg::DReg(D(6)), Size::Word), Addressing::AddrDisplacement(A(3), 0x3c))
        );
        test!(
            [0x16f2, 0x2000],
            Instruction::Move(Size::Byte, Addressing::AddrIndex(0, A(2), IndexReg::DReg(D(2)), Size::Word), Addressing::AddrPostIncrement(A(3)))
        );
        test!(
            [0x1732, 0x2000],
            Instruction::Move(Size::Byte, Addressing::AddrIndex(0, A(2), IndexReg::DReg(D(2)), Size::Word), Addressing::AddrPreDecrement(A(3)))
        );
        test!(
            [0x11fb, 0x305e, 0xe446],
            Instruction::Move(Size::Byte, Addressing::PcIndex(0x5e, IndexReg::DReg(D(3)), Size::Word), Addressing::AbsoluteShort(0xe446))
        );
        test!(
            [0x143b, 0x005e],
            Instruction::Move(Size::Byte, Addressing::PcIndex(0x5e, IndexReg::DReg(D(0)), Size::Word), Addressing::DReg(D(2)))
        );
        test!(
            [0x12bb, 0x005e],
            Instruction::Move(Size::Byte, Addressing::PcIndex(0x5e, IndexReg::DReg(D(0)), Size::Word), Addressing::Addr(A(1)))
        );
        test!(
            [0x137b, 0x005e, 0x0022],
            Instruction::Move(Size::Byte, Addressing::PcIndex(0x5e, IndexReg::DReg(D(0)), Size::Word), Addressing::AddrDisplacement(A(1), 0x22))
        );
        test!(
            [0x46fc, 0x2700],
            Instruction::MoveToSr(Addressing::ImmediateWord(0x2700))
        );
    }
    #[test]
    fn movea() {
        test!(
            [0x387c, 0x6000],
            Instruction::Movea(Size::Word, Addressing::ImmediateWord(0x6000), A(4))
        );
        test!(
            [0x3878, 0xee4a],
            Instruction::Movea(Size::Word, Addressing::AbsoluteShort(0xee4a), A(4))
        );
        test!(
            [0x3841],
            Instruction::Movea(Size::Word, Addressing::DReg(D(1)), A(4))
        );
        test!(
            [0x2c4a],
            Instruction::Movea(Size::Long, Addressing::AReg(A(2)), A(6))
        );
        test!(
            [0x3253],
            Instruction::Movea(Size::Word, Addressing::Addr(A(3)), A(1))
        );
        test!(
            [0x326b, 0x002c],
            Instruction::Movea(Size::Word, Addressing::AddrDisplacement(A(3), 0x2c), A(1))
        );
        test!(
            [0x2459],
            Instruction::Movea(Size::Long, Addressing::AddrPostIncrement(A(1)), A(2))
        );
        test!(
            [0x2461],
            Instruction::Movea(Size::Long, Addressing::AddrPreDecrement(A(1)), A(2))
        );
        test!(
            [0x2457],
            Instruction::Movea(Size::Long, Addressing::Addr(A(7)), A(2))
        );
        test!(
            [0x245f],
            Instruction::Movea(Size::Long, Addressing::AddrPostIncrement(A(7)), A(2))
        );
        test!(
            [0x2467],
            Instruction::Movea(Size::Long, Addressing::AddrPreDecrement(A(7)), A(2))
        );
        test!(
            [0x2674, 0x0000],
            Instruction::Movea(Size::Long, Addressing::AddrIndex(0x00, A(4), IndexReg::DReg(D(0)), Size::Word), A(3))
        );
        test!(
            [0x2674, 0x0018],
            Instruction::Movea(Size::Long, Addressing::AddrIndex(0x18, A(4), IndexReg::DReg(D(0)), Size::Word), A(3))
        );
    }
    #[test]
    fn clr() {
        test!(
            [0x4242],
            Instruction::Clr(Size::Word, Addressing::DReg(D(2)))
        );
        test!(
            [0x4280],
            Instruction::Clr(Size::Long, Addressing::DReg(D(0)))
        );
        test!(
            [0x4278, 0x8500],
            Instruction::Clr(Size::Word, Addressing::AbsoluteShort(0x8500))
        );
        test!(
            [0x4228, 0x003c],
            Instruction::Clr(Size::Byte, Addressing::AddrDisplacement(A(0), 0x3c))
        );
        test!(
            [0x4268, 0x001a],
            Instruction::Clr(Size::Word, Addressing::AddrDisplacement(A(0), 0x1a))
        );
        test!(
            [0x4291],
            Instruction::Clr(Size::Long, Addressing::Addr(A(1)))
        );
        test!(
            [0x42a9, 0x0004],
            Instruction::Clr(Size::Long, Addressing::AddrDisplacement(A(1), 0x4))
        );
        test!(
            [0x4299],
            Instruction::Clr(Size::Long, Addressing::AddrPostIncrement(A(1)))
        );
        test!(
            [0x42a1],
            Instruction::Clr(Size::Long, Addressing::AddrPreDecrement(A(1)))
        );
    }
    #[test]
    fn neg() {
        test!(
            [0x4478, 0xfe26],
            Instruction::Neg(Size::Word, Addressing::AbsoluteShort(0xfe26))
        );
        test!(
            [0x4480],
            Instruction::Neg(Size::Long, Addressing::DReg(D(0)))
        );
        test!(
            [0x4451],
            Instruction::Neg(Size::Word, Addressing::Addr(A(1)))
        );
        test!(
            [0x4469, 0x001a],
            Instruction::Neg(Size::Word, Addressing::AddrDisplacement(A(1), 0x1a))
        );
        test!(
            [0x4459],
            Instruction::Neg(Size::Word, Addressing::AddrPostIncrement(A(1)))
        );
        test!(
            [0x4461],
            Instruction::Neg(Size::Word, Addressing::AddrPreDecrement(A(1)))
        );
    }
    #[test]
    fn not() {
        test!(
            [0x4678, 0xfe26],
            Instruction::Not(Size::Word, Addressing::AbsoluteShort(0xfe26))
        );
        test!(
            [0x4680],
            Instruction::Not(Size::Long, Addressing::DReg(D(0)))
        );
        test!(
            [0x4651],
            Instruction::Not(Size::Word, Addressing::Addr(A(1)))
        );
        test!(
            [0x4669, 0x001a],
            Instruction::Not(Size::Word, Addressing::AddrDisplacement(A(1), 0x1a))
        );
        test!(
            [0x4659],
            Instruction::Not(Size::Word, Addressing::AddrPostIncrement(A(1)))
        );
        test!(
            [0x4661],
            Instruction::Not(Size::Word, Addressing::AddrPreDecrement(A(1)))
        );
    }
    #[test]
    fn swap() {
        test!(
            [0x4840],
            Instruction::Swap(D(0))
        );
    }
    #[test]
    fn ext() {
        test!(
            [0x4880],
            Instruction::Ext(Size::Word, D(0))
        );
        test!(
            [0x48c1],
            Instruction::Ext(Size::Long, D(1))
        );
    }
    #[test]
    fn pea() {
        test!(
            [0x4878, 0xe53c],
            Instruction::Pea(Addressing::AbsoluteShort(0xe53c))
        );
        test!(
            [0x4852],
            Instruction::Pea(Addressing::Addr(A(2)))
        );
        test!(
            [0x4868, 0x002a],
            Instruction::Pea(Addressing::AddrDisplacement(A(0), 0x2a))
        );
    }
    #[test]
    fn illegal() {
        test!(
            [0x4afc],
            Instruction::Illegal
        );
    }
    #[test]
    fn tas() {
        test!(
            [0x4af8, 0xfe00],
            Instruction::Tas(Addressing::AbsoluteShort(0xfe00))
        );
        test!(
            [0x4ac1],
            Instruction::Tas(Addressing::DReg(D(1)))
        );
        test!(
            [0x4ad2],
            Instruction::Tas(Addressing::Addr(A(2)))
        );
        test!(
            [0x4ae8, 0x002a],
            Instruction::Tas(Addressing::AddrDisplacement(A(0), 0x2a))
        );
    }
    #[test]
    fn tst() {
        test!(
            [0x4a04],
            Instruction::Tst(Size::Byte, Addressing::DReg(D(4)))
        );
        test!(
            [0x4a81],
            Instruction::Tst(Size::Long, Addressing::DReg(D(1)))
        );
        test!(
            [0x4a38, 0xaa80],
            Instruction::Tst(Size::Byte, Addressing::AbsoluteShort(0xaa80))
        );
        test!(
            [0x4a78, 0xaa80],
            Instruction::Tst(Size::Word, Addressing::AbsoluteShort(0xaa80))
        );
        test!(
            [0x4a79, 0x00a1, 0x000c],
            Instruction::Tst(Size::Word, Addressing::AbsoluteWord(0xa1000c))
        );
        test!(
            [0x4ab9, 0x00a1, 0x0008],
            Instruction::Tst(Size::Long, Addressing::AbsoluteWord(0xa10008))
        );
        test!(
            [0x4a54],
            Instruction::Tst(Size::Word, Addressing::Addr(A(4)))
        );
        test!(
            [0x4a69, 0x001a],
            Instruction::Tst(Size::Word, Addressing::AddrDisplacement(A(1), 0x1a))
        );
    }
    #[test]
    fn trap() {
        test!(
            [0x4e41],
            Instruction::Trap(1)
        );
    }
    #[test]
    fn trapv() {
        test!(
            [0x4e76],
            Instruction::Trapv
        );
    }
    #[test]
    fn link() {
        test!(
            [0x4e54, 0x1087],
            Instruction::Link(A(4), 0x1087)
        );
    }
    #[test]
    fn unlk() {
        test!(
            [0x4e58],
            Instruction::Unlk(A(0))
        );
    }
    #[test]
    fn nop() {
        test!(
            [0x4e71],
            Instruction::Nop
        );
    }
    #[test]
    fn stop() {
        test!(
            [0x4e72, 0x2500],
            Instruction::Stop(0x2500)
        );
    }
    #[test]
    fn rte() {
        test!(
            [0x4e73],
            Instruction::Rte
        );
    }
    #[test]
    fn rtr() {
        test!(
            [0x4e77],
            Instruction::Rtr
        );
    }
    #[test]
    fn rts() {
        test!(
            [0x4e75],
            Instruction::Rts
        );
    }
    #[test]
    fn jsr() {
        test!(
            [0x4eb9, 0x0004, 0xb98c],
            Instruction::Jsr(Addressing::AbsoluteWord(0x4b98c))
        );
    }
    #[test]
    fn jmp() {
        test!(
            [0x4ed1],
            Instruction::Jmp(Addressing::Addr(A(1)))
        );
        test!(
            [0x4ee9, 0x0010],
            Instruction::Jmp(Addressing::AddrDisplacement(A(1), 0x10))
        );
        test!(
            [0x4ef9, 0x0006, 0x5a70],
            Instruction::Jmp(Addressing::AbsoluteWord(0x65a70))
        );
    }
    #[test]
    fn lea() {
        test!(
            [0x41f8, 0xfff4],
            Instruction::Lea(Addressing::AbsoluteShort(0xfff4), A(0))
        );
        test!(
            [0x43f9, 0xffff, 0x7cc0],
            Instruction::Lea(Addressing::AbsoluteWord(0xffff_7cc0), A(1))
        );
        test!(
            [0x45d6],
            Instruction::Lea(Addressing::Addr(A(6)), A(2))
        );
        test!(
            [0x43e9, 0x0130],
            Instruction::Lea(Addressing::AddrDisplacement(A(1), 0x130), A(1))
        );
        test!(
            [0x45f3, 0x3000],
            Instruction::Lea(Addressing::AddrIndex(0x0, A(3), IndexReg::DReg(D(3)), Size::Word), A(2))
        );
    }
    #[test]
    fn scc() {
        test!(
            [0x50f8, 0xf000],
            Instruction::S(Condition::True, Addressing::AbsoluteShort(0xf000))
        );
        test!(
            [0x51f8, 0xf000],
            Instruction::S(Condition::False, Addressing::AbsoluteShort(0xf000))
        );
        test!(
            [0x52f8, 0xf000],
            Instruction::S(Condition::Higher, Addressing::AbsoluteShort(0xf000))
        );
        test!(
            [0x53f8, 0xf000],
            Instruction::S(Condition::LowerOrSame, Addressing::AbsoluteShort(0xf000))
        );
        test!(
            [0x54f8, 0xf000],
            Instruction::S(Condition::CarryClear, Addressing::AbsoluteShort(0xf000))
        );
        test!(
            [0x55f8, 0xf000],
            Instruction::S(Condition::CarrySet, Addressing::AbsoluteShort(0xf000))
        );
        test!(
            [0x56f8, 0xf000],
            Instruction::S(Condition::NotEqual, Addressing::AbsoluteShort(0xf000))
        );
        test!(
            [0x57f8, 0xf000],
            Instruction::S(Condition::Equal, Addressing::AbsoluteShort(0xf000))
        );
        test!(
            [0x58f8, 0xf000],
            Instruction::S(Condition::OverflowClear, Addressing::AbsoluteShort(0xf000))
        );
        test!(
            [0x59f8, 0xf000],
            Instruction::S(Condition::OverflowSet, Addressing::AbsoluteShort(0xf000))
        );
        test!(
            [0x5af8, 0xf000],
            Instruction::S(Condition::Plus, Addressing::AbsoluteShort(0xf000))
        );
        test!(
            [0x5bf8, 0xf000],
            Instruction::S(Condition::Minus, Addressing::AbsoluteShort(0xf000))
        );
        test!(
            [0x5cf8, 0xf000],
            Instruction::S(Condition::GreaterOrEqual, Addressing::AbsoluteShort(0xf000))
        );
        test!(
            [0x5df8, 0xf000],
            Instruction::S(Condition::LessThan, Addressing::AbsoluteShort(0xf000))
        );
        test!(
            [0x5ef8, 0xf000],
            Instruction::S(Condition::GreaterThan, Addressing::AbsoluteShort(0xf000))
        );
        test!(
            [0x5ff8, 0xf000],
            Instruction::S(Condition::LessOrEqual, Addressing::AbsoluteShort(0xf000))
        );
    }
    #[test]
    fn dbcc() {
        test!(
            [0x51c8, 0x556a],
            Instruction::Db(Condition::False, D(0), 0x556a)
        );
        test!(
            [0x51cb, 0xffa0],
            Instruction::Db(Condition::False, D(3), 0xffa0)
        );
        test!(
            [0x57c9, 0xfffc],
            Instruction::Db(Condition::Equal, D(1), 0xfffc)
        );
        test!(
            [0x5bcc, 0xffdc],
            Instruction::Db(Condition::Minus, D(4), 0xffdc)
        );
    }
    #[test]
    fn branches() {
        test!(
            [0x6024],
            Instruction::Bra(0x24)
        );
        test!(
            [0x6000, 0x4e1a],
            Instruction::Bra(0x4e1a)
        );
        test!(
            [0x6110],
            Instruction::Bsr(0x10)
        );
        test!(
            [0x6510],
            Instruction::B(Condition::CarrySet, 0x10)
        );
    }
    #[test]
    fn moveq() {
        test!(
            [0x7280],
            Instruction::Moveq(0x80, D(1))
        );
    }
    #[test]
    fn divs() {
        test!(
            [0x81fc, 0x000a],
            Instruction::Divs(Addressing::ImmediateWord(0xa), D(0))
        );
    }
    #[test]
    fn divu() {
        test!(
            [0x82f8, 0xf314],
            Instruction::Divu(Addressing::AbsoluteShort(0xf314), D(1))
        );
        test!(
            [0x82c1],
            Instruction::Divu(Addressing::DReg(D(1)), D(1))
        );
        test!(
            [0x82d1],
            Instruction::Divu(Addressing::Addr(A(1)), D(1))
        );
        test!(
            [0x82e8, 0x0004],
            Instruction::Divu(Addressing::AddrDisplacement(A(0), 4), D(1))
        );
        test!(
            [0x82d9],
            Instruction::Divu(Addressing::AddrPostIncrement(A(1)), D(1))
        );
        test!(
            [0x82e1],
            Instruction::Divu(Addressing::AddrPreDecrement(A(1)), D(1))
        );
    }
}
