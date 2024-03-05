use core::convert::{TryFrom, TryInto};

use crate::arch::la64::{
    register::{MachineError, TLBRERA},
    MErrCtl,
};

use bit_field::BitField;

use super::ecfg;

impl_define_csr!(
    EStat,
    "Record the status information of the exceptions,
including the first(`Ecode`) and second level(`EsubCode`) encoding of the triggered exceptions,
and the status of each interrupt."
);
impl_read_csr!(0x5, EStat);
impl_write_csr!(0x5, EStat);

impl EStat {
    fn get_is(&self) -> usize {
        self.get_bits(0..=12).bits
    }
    pub fn is_interrupt_happened(&self, index: Interrupt) -> bool {
        // 0-12位为中断
        self.bits.get_bit(index.try_into().unwrap())
    }
    // 只有写0和1位有效，这两位控制软件中断
    pub fn set_sw_int(&mut self, is_sw_int: bool, value: bool) -> &mut Self {
        self.bits.set_bit(is_sw_int as usize, value);
        self
    }
    // 例外类型一级编码。触发例外时：
    // 如果是 TLB 重填例外或机器错误例外，该域保持不变；
    // 否则，硬件会根据例外类型将表 7- 8 中 Ecode 栏定义的数值写入该域。
    //例外类型一级编号 21-16位
    fn ecode(&self) -> usize {
        self.bits.get_bits(16..=21)
    }
    //例外类型二级编号 22-30位
    pub fn exception_sub_code(&self) -> usize {
        self.bits.get_bits(22..=30)
    }

    pub fn cause(&self) -> Trap {
        let is_tlb_reload = false && TLBRERA::read().is_tlbr();
        if is_tlb_reload {
            return Trap::TLBReFill;
        } else if MErrCtl::read().is_merr() {
            return Trap::MachineError(MachineError::CacheCheckError);
        }
        let ecode = self.ecode();
        if ecode == 0 {
            if ecfg::ECfg::read().get_entries_margin() == 0 {
                return Trap::Interrupt(
                    Interrupt::try_from(63 - self.get_is().leading_zeros() as usize).unwrap(),
                );
            }
        } else {
            return Trap::Exception(Exception::try_from(ecode).unwrap());
        }

        unreachable!()
    }
}

// 异常类型
#[derive(Debug, Clone, Copy, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
#[repr(usize)]
pub enum Exception {
    /// This exception is triggered when the virtual address of a LOAD(i.e. `ld.{b,h,w,d}`) operation finds a match in the TLB with `V=0`.
    PageInvalidLoad = 0x1,
    /// This exception is triggered when the virtual address of a STORE(i.e. `st.{b,h,w,d}`) operation finds a match in the TLB with `V=0`.
    PageInvalidStore = 0x2,
    /// This exception is triggered when the virtual address of an instruction fetching  operation finds a match in the TLB with `V=0`.
    PageInvalidFetch = 0x3,
    /// the virtual address of a store operation matches a TLB entry with `V=1`, `D=0` and a permitted privilege.
    PageModifyFault = 0x4,
    /// the virtual address of a load operation matches a TLB entry with `V=1`, `NR=1` and a permitted privilege.
    PageNonReadableFault = 0x5,
    /// the virtual address of a fetch operation matches a TLB entry with `V=1`, `NX=1` and a permitted privilege.
    PageNonExecutableFault = 0x6,
    PagePrivilegeIllegal = 0x7,
    #[doc = " When the program has a functional error that causes the address of the instruction fetch,
 or memory access instruction to appear illegal
(such as the instruction fetch address is not aligned on 4-byte boundaries,
and the privileged address space is accessed),
ADdress error Exception for Fetching instructions (ADEF),
or ADdress error Exception for Memory access instructions (ADEM) will be triggered."]
    AddressError = 0x8,
    AddressNotAligned = 0x9,           //地址不对齐
    BoundsCheckFault = 0xA,            //越界检查错误
    Syscall = 0xB,                     //系统调用
    Breakpoint = 0xC,                  //调试中断
    InstructionNonDefined = 0xD,       //指令不合规
    InstructionPrivilegeIllegal = 0xE, //特权指令不合规
    FloatingPointUnavailable = 0xF,
}

// 中断类型
#[derive(Debug, Clone, Copy, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
#[repr(usize)]
pub enum Interrupt {
    ///Software Interrupt 0
    SWI0 = 0,
    ///Software Interrupt 1
    SWI1,
    ///Hardware Interrupt 0
    HWI0,
    ///Hardware Interrupt 1
    HWI1,
    ///Hardware Interrupt 2
    HWI2,
    ///Hardware Interrupt 3
    HWI3,
    ///Hardware Interrupt 4
    HWI4,
    ///Hardware Interrupt 5
    HWI5,
    ///Hardware Interrupt 6
    HWI6,
    ///Hardware Interrupt 7
    HWI7,
    ///Performance Monitor Counter Overflow Interrupt
    PMCOV,
    ///Timer Interrupt
    Timer,
    ///Inter-Processor Interrupt
    IPI,
}

#[derive(Debug, Clone, Copy)]
pub enum Trap {
    Exception(Exception),
    Interrupt(Interrupt),
    MachineError(MachineError),
    TLBReFill,
    Unknown,
}
impl Trap {
    pub fn is_tlb_refill(&self) -> bool {
        if let Self::TLBReFill = self {
            true
        } else {
            false
        }
    }
    pub fn is_machine_error(&self) -> bool {
        if let Self::MachineError(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_exception(&self) -> bool {
        if let Self::Exception(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_timer(&self) -> bool {
        if let Self::Interrupt(Interrupt::Timer) = self {
            true
        } else {
            false
        }
    }
    pub fn is_syscall(&self) -> bool {
        if let Self::Exception(Exception::Syscall) = self {
            true
        } else {
            false
        }
    }
}
