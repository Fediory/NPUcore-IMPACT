use core::fmt::Debug;

use crate::{
    arch::la64::register::PrMd,
    task::{SignalStack, Signals},
};

/// General registers
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct GeneralRegs {
    pub pc: usize,
    pub ra: usize,
    pub tp: usize,
    pub sp: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    pub t7: usize,
    pub t8: usize,
    pub r21: usize,
    pub fp: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
}

impl core::ops::Index<usize> for GeneralRegs {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        let gp = unsafe { (self as *const _ as *const [usize; 32]).as_ref().unwrap() };
        if index != 0 {
            &gp[index]
        } else {
            &0
        }
    }
}
impl core::ops::IndexMut<usize> for GeneralRegs {
    fn index_mut(&mut self, index: usize) -> &mut usize {
        assert!(index != 0);
        let gp = unsafe { (self as *mut _ as *mut [usize; 32]).as_mut().unwrap() };
        &mut gp[index]
    }
}

impl Debug for GeneralRegs {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GeneralRegs")
            .field("pc", &format_args!("{:#x}", self.pc))
            .field("ra", &format_args!("{:#x}", self.ra))
            .field("tp", &format_args!("{:#x}", self.tp))
            .field("sp", &format_args!("{:#x}", self.sp))
            .field("a0", &self.a0)
            .field("a1", &self.a1)
            .field("a2", &self.a2)
            .field("a3", &self.a3)
            .field("a4", &self.a4)
            .field("a5", &self.a5)
            .field("a6", &self.a6)
            .field("a7", &self.a7)
            .field("t0", &self.t0)
            .field("t1", &self.t1)
            .field("t2", &self.t2)
            .field("t3", &self.t3)
            .field("t4", &self.t4)
            .field("t5", &self.t5)
            .field("t6", &self.t6)
            .field("t7", &self.t7)
            .field("t8", &self.t8)
            .field("r21", &self.r21)
            .field("fp", &format_args!("{:#x}", self.fp))
            .field("s0", &self.s0)
            .field("s1", &self.s1)
            .field("s2", &self.s2)
            .field("s3", &self.s3)
            .field("s4", &self.s4)
            .field("s5", &self.s5)
            .field("s6", &self.s6)
            .field("s7", &self.s7)
            .field("s8", &self.s8)
            .finish()
    }
}
/// FP registers
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct FloatRegs {
    pub f: [usize; 32],
    pub fcsr: u32,
    pub fcc: u8,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct MachineContext {
    gp: GeneralRegs,
    fp: FloatRegs,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UserContext {
    pub flags: usize,
    pub link: usize,
    pub stack: SignalStack,
    pub sigmask: Signals,
    pub __pad: [u8; 128],
    pub mcontext: MachineContext,
}

impl UserContext {
    pub const PADDING_SIZE: usize = 128;
}

#[repr(C)]
#[derive(Clone, Copy)]
/// The trap cotext containing the user context and the supervisor level
pub struct TrapContext {
    /// The registers to be preserved.
    pub gp: GeneralRegs,
    pub fp: FloatRegs,
    /// A copy of register a0, useful when we need to restart syscall
    pub origin_a0: usize,
    /// Privilege level of the trap context
    pub sstatus: PrMd,
    /// Supervisor Address Translation and Protection
    pub kernel_satp: usize,
    /// The pointer to trap_handler
    pub trap_handler: usize,
    /// The current sp to be recovered on next entry into kernel space.
    pub kernel_sp: usize,
}
impl Debug for TrapContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TrapContext")
            .field("gp", &self.gp)
            .field("fp", &self.fp)
            .field("origin_a0", &self.origin_a0)
            .field("sstatus", &self.sstatus)
            .field("kernel_satp", &format_args!("{:#x}", self.kernel_satp))
            .field("trap_handler", &format_args!("{:#x}", self.trap_handler))
            .field("kernel_sp", &format_args!("{:#x}", self.kernel_sp))
            .finish()
    }
}
impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.gp.sp = sp;
    }
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let pr_md = PrMd::read();
        let mut cx = Self {
            gp: GeneralRegs::default(),
            fp: FloatRegs::default(),
            origin_a0: 0,
            sstatus: pr_md,
            kernel_satp,
            trap_handler,
            kernel_sp,
        };
        cx.gp.pc = entry;
        cx.set_sp(sp);
        cx
    }
}
