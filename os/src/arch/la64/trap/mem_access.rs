use alloc::collections::btree_set::Intersection;
use bit_field::BitField;
use core::{convert::TryInto, fmt::Debug};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use super::context::GeneralRegs;

#[derive(Clone, Copy)]
pub struct Instruction {
    bits: u32,
}
impl From<u32> for Instruction {
    fn from(value: u32) -> Self {
        Self { bits: value }
    }
}
impl From<*const Instruction> for Instruction {
    fn from(value: *const Instruction) -> Self {
        unsafe { *value }
    }
}
impl Debug for Instruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Instruction")
            .field("", &self.get_op_code().unwrap())
            .field("", &self.get_rd_num())
            .finish()
    }
}
impl Instruction {
    #[allow(unused)]
    pub fn get_op_code(&self) -> Result<OpCode, num_enum::TryFromPrimitiveError<OpCode>> {
        (self.bits >> (32 - 8)).try_into().or_else(|_| {
            (self.bits >> (32 - 10))
                .try_into()
                .or_else(|_| (self.bits >> (32 - 17)).try_into())
        })
    }
    pub fn get_addr(
        &self,
        gp: &GeneralRegs,
    ) -> Result<usize, num_enum::TryFromPrimitiveError<OpCode>> {
        self.get_op_code().map(|op| match op {
            // si14, rj, rd
            OpCode::LdPtrW | OpCode::StPtrW | OpCode::LdPtrD | OpCode::StPtrD => {
                let mut rj = self.get_rj_num();
                rj = if rj != 0 { gp[rj] } else { 0 };
                rj + ((self.get_si14() << 2) as u16 as i16 as usize)
            }
            // si12, rj, rd
            OpCode::LdB
            | OpCode::LdH
            | OpCode::LdW
            | OpCode::LdD
            | OpCode::StB
            | OpCode::StH
            | OpCode::StW
            | OpCode::StD
            | OpCode::LdBU
            | OpCode::LdHU
            | OpCode::LdWU
            | OpCode::FLdS
            | OpCode::FStS
            | OpCode::FLdD
            | OpCode::FStD => {
                let mut rj = self.get_rj_num();
                rj = if rj != 0 { gp[rj] } else { 0 };
                rj + ((self.get_si12() << 4) as u16 as i16 as usize >> 4)
            }

            // rk, rj, rd
            OpCode::LdXB
            | OpCode::LdXH
            | OpCode::LdXW
            | OpCode::LdXD
            | OpCode::StXB
            | OpCode::StXH
            | OpCode::StXW
            | OpCode::StXD
            | OpCode::LdXBU
            | OpCode::LdXHU
            | OpCode::LdXWU
            | OpCode::FLdXS
            | OpCode::FLdXD
            | OpCode::FStXS
            | OpCode::FStXD => {
                let mut rj = self.get_rj_num();
                let mut rk = self.get_rk_num();
                rj = if rj != 0 { gp[rj] } else { 0 };
                rk = if rk != 0 { gp[rk] } else { 0 };
                rj + rk
            }
        })
    }
    #[allow(unused)]
    pub fn get_rd_num(&self) -> usize {
        let beg = 0;
        self.bits.get_bits(beg..beg + 5) as usize
    }
    #[allow(unused)]
    pub fn get_rj_num(&self) -> usize {
        let beg = 5;
        self.bits.get_bits(beg..beg + 5) as usize
    }
    #[allow(unused)]
    pub fn get_rk_num(&self) -> usize {
        let beg = 10;
        self.bits.get_bits(beg..beg + 5) as usize
    }
    #[allow(unused)]
    pub fn get_si12(&self) -> usize {
        let beg = 10;
        self.bits.get_bits(beg..beg + 12) as usize
    }
    #[allow(unused)]
    pub fn get_si14(&self) -> usize {
        let beg = 10;
        self.bits.get_bits(beg..beg + 14) as usize
    }
}
// 8, 10, 17 (17-3),
#[repr(u32)]
#[derive(TryFromPrimitive, IntoPrimitive, Debug)]
pub enum OpCode {
    LdPtrW = 0b0010_0100,
    StPtrW = 0b0010_0101,
    LdPtrD = 0b0010_0110,
    StPtrD = 0b0010_0111,
    LdB = 0b0010_1000_00,
    LdH = 0b0010_1000_01,
    LdW = 0b0010_1000_10,
    LdD = 0b0010_1000_11,
    StB = 0b0010_1001_00,
    StH = 0b0010_1001_01,
    StW = 0b0010_1001_10,
    StD = 0b0010_1001_11,
    LdBU = 0b0010_1010_00,
    LdHU = 0b0010_1010_01,
    LdWU = 0b0010_1010_10,
    FLdS = 0b0010_1011_00,
    FStS = 0b0010_1011_01,
    FLdD = 0b0010_1011_10,
    FStD = 0b0010_1011_11,
    LdXB = 0b0011_1000_0000_00_000,
    LdXH = 0b0011_1000_0000_01_000,
    LdXW = 0b0011_1000_0000_10_000,
    LdXD = 0b0011_1000_0000_11_000,
    StXB = 0b0011_1000_0001_00_000,
    StXH = 0b0011_1000_0001_01_000,
    StXW = 0b0011_1000_0001_10_000,
    StXD = 0b0011_1000_0001_11_000,
    LdXBU = 0b0011_1000_0010_00_000,
    LdXHU = 0b0011_1000_0010_01_000,
    LdXWU = 0b0011_1000_0010_10_000,
    FLdXS = 0b0011_1000_0011_00_000,
    FLdXD = 0b0011_1000_0011_01_000,
    FStXS = 0b0011_1000_0011_10_000,
    FStXD = 0b0011_1000_0011_11_000,
}
impl OpCode {
    pub fn is_store(&self) -> bool {
        match self {
            OpCode::LdPtrW
            | OpCode::LdPtrD
            | OpCode::LdB
            | OpCode::LdH
            | OpCode::LdW
            | OpCode::LdD
            | OpCode::LdBU
            | OpCode::LdHU
            | OpCode::LdWU
            | OpCode::FLdS
            | OpCode::FLdD
            | OpCode::LdXB
            | OpCode::LdXH
            | OpCode::LdXW
            | OpCode::LdXD
            | OpCode::LdXBU
            | OpCode::LdXHU
            | OpCode::LdXWU
            | OpCode::FLdXS
            | OpCode::FLdXD => false,

            OpCode::StPtrW
            | OpCode::StPtrD
            | OpCode::StB
            | OpCode::StH
            | OpCode::StW
            | OpCode::StD
            | OpCode::FStS
            | OpCode::FStD
            | OpCode::StXB
            | OpCode::StXH
            | OpCode::StXW
            | OpCode::StXD
            | OpCode::FStXS
            | OpCode::FStXD => true,
        }
    }
    pub fn get_size(&self) -> usize {
        match self {
            OpCode::LdPtrW => 4,
            OpCode::StPtrW => 4,
            OpCode::LdPtrD => 8,
            OpCode::StPtrD => 8,
            OpCode::LdB => 1,
            OpCode::LdH => 2,
            OpCode::LdW => 4,
            OpCode::LdD => 8,
            OpCode::StB => 1,
            OpCode::StH => 2,
            OpCode::StW => 4,
            OpCode::StD => 8,
            OpCode::LdBU => 1,
            OpCode::LdHU => 2,
            OpCode::LdWU => 4,
            OpCode::FLdS => 4,
            OpCode::FStS => 4,
            OpCode::FLdD => 8,
            OpCode::FStD => 8,
            OpCode::LdXB => 1,
            OpCode::LdXH => 2,
            OpCode::LdXW => 4,
            OpCode::LdXD => 8,
            OpCode::StXB => 1,
            OpCode::StXH => 2,
            OpCode::StXW => 4,
            OpCode::StXD => 8,
            OpCode::LdXBU => 1,
            OpCode::LdXHU => 2,
            OpCode::LdXWU => 4,
            OpCode::FLdXS => 4,
            OpCode::FLdXD => 8,
            OpCode::FStXS => 4,
            OpCode::FStXD => 8,
        }
    }
    pub fn is_float_op(&self) -> bool {
        match self {
            OpCode::FLdS
            | OpCode::FStS
            | OpCode::FLdD
            | OpCode::FStD
            | OpCode::FLdXS
            | OpCode::FLdXD
            | OpCode::FStXS
            | OpCode::FStXD => true,
            _ => false,
        }
    }
    pub fn is_unsigned_ld(&self) -> bool {
        match self {
            OpCode::LdBU
            | OpCode::LdHU
            | OpCode::LdWU
            | OpCode::LdXBU
            | OpCode::LdXHU
            | OpCode::LdXWU => true,
            _ => false,
        }
    }
}
