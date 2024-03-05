use bit_field::BitField;
use core::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
    ptr::addr_of,
};
use log::debug;

use crate::{arch::board::DMA_ADDR, task};
const DMA_ORDER: usize = DMA_ADDR;
#[allow(unused)]
const DMA_ORDER_ADDR_LOW: usize = 0;
#[allow(unused)]
const DMA_SADDR: usize = 0x4;
#[allow(unused)]
const DMA_DADDR: usize = 0x8;
#[allow(unused)]
const DMA_LENGTH: usize = 0xc;
#[allow(unused)]
const DMA_STEP_LENGTH: usize = 0x10;
#[allow(unused)]
const DMA_STEP_TIMES: usize = 0x14;
#[allow(unused)]
const DMA_CMD: usize = 0x18;
#[allow(unused)]
const DMA_ORDER_ADDR_HIGH: usize = 0x20;
#[allow(unused)]
const DMA_SADDR_HIGH: usize = 0x24;
#[allow(unused)]
const DMA_RETRY_TIMES: usize = 10;

impl_define_mem_reg!(
    DMAOrderAddrLow,
    DMA_ORDER_ADDR_LOW,
    "下一个描述符低位地址寄存器"
);
impl_define_mem_reg!(DMACmd, DMA_CMD, "控制寄存器");
impl Default for DMACmd {
    fn default() -> Self {
        Self {
            bits: Default::default(),
        }
    }
}
impl Default for DMAOrderAddrLow {
    fn default() -> Self {
        Self {
            bits: Default::default(),
        }
    }
}
#[allow(unused)]
impl DMAOrderAddrLow {
    impl_get_set!(get_ac97_wr, set_ac97_wr, 31);
    impl_get_set!(get_ac97_stero, set_ac97_stero, 30);
    impl_get_set_field!(get_ac97_wr_byte_log, set_ac97_wr_byte_log, 28..=29);
    impl_get_set_field!(get_dma_addr, set_dma_addr, 0..=27);
}

#[allow(unused)]
impl DMACmd {
    impl_get_set_field!(get_cmd, set_cmd, 13..=14);
    impl_get_set!(is_send, set_send, 12);
    impl_get_set_enum!(get_write_state, set_write_state, 8..=11, WriteStatus);
    impl_get_set_enum!(get_read_state, set_read_state, 4..=7, ReadStatus);
    impl_get_set!(get_trans_over, set_trans_over, 3);
    impl_get_set!(get_single_trans_over, set_single_trans_over, 2);
    impl_get_set!(get_interrupt, set_interrupt, 1);
    impl_get_set!(get_interrupt_mask, set_interrupt_mask, 0);
}

#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive, Debug)]
#[repr(usize)]
pub enum ReadStatus {
    /// 读状态正处于空闲状态
    ReadIdle = 0x0,
    /// 接收到开始 dma 操作的 start 信号后，进入准备好状态，开始读描述符
    ReadReady = 0x1,
    /// 向内存发出读描述符请求，等待内存应答
    GetOrder = 0x2,
    /// 内存接收读描述符请求，正在执行读操作
    ReadOrder = 0x3,
    /// 内存接收 dma 读数据请求，正在执行读数据操作
    FinishOrderEnd = 0x4,
    /// DMA 向内存发出读数据请求，等待内存应答
    RDdrWait = 0x5,
    ///内存接收 dma 读数据请求，正在执行读数据操作
    ReadDdr = 0x6,
    /// 内存完成 dma 的一次读数据请求
    ReadDdrEnd = 0x7,
    /// DMA 进入读设备状态
    ReadDev = 0x8,
    /// 设备返回读数据，结束此次读设备请求
    ReadDevEnd = 0x9,
    /// 结束一次 step 操作，step times 减 1
    ReadStepEnd = 0xa,
    Reserved = 0xb,
}
#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive, Debug)]
#[repr(usize)]
pub enum WriteStatus {
    /// 写状态正处于空闲状态
    WriteIdle = 0x0,
    /// DMA 判断需要执行读设备写内存操作，并发起写内存请求，但是内存还没准备好响应请求，因此 dma 一直在等待内存的响应
    WDdrWait = 0x1,
    /// 内存接收了 dma 写请求，但是还没有执行完写操作
    WriteDdr = 0x2,
    /// 内存接收了 dma 写请求，并完成写操作，此时 dma 处于写内存操作完成状态
    WriteDdrEnd = 0x3,
    /// DMA 发出将 dma 状态寄存器写回内存的请求，等待内存接收请求
    WriteDMAWait = 0x4,
    /// 内存接收写 dma 状态请求，但是操作还未完成
    WriteDMA = 0x5,
    /// 内存完成写 dma 状态操作
    WriteDMAEnd = 0x6,
    /// DMA 完成一次 length 长度的操作（也就是说完成一个 step）
    WriteStepEnd = 0x7,
    Reserved = 0x8,
}
#[repr(C, align(32))]
#[derive(Clone, Copy, Default)]
pub struct DMADesc {
    dma_order_addr_low: DMAOrderAddrLow,
    dma_saddr: u32,
    dma_daddr: u32,
    dma_length: u32,
    dma_step_length: u32,
    dma_step_times: u32,
    dma_cmd: DMACmd,
    reserved: u32,
    dma_order_addr_high: u32,
    dma_saddr_high: u32,
    _align: [u32; 2],
}
impl Debug for DMADesc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DMADesc")
            .field(
                "dma_order_addr",
                &format_args!("{:#x}", self.get_dma_order_addr()),
            )
            .field("dma_saddr", &format_args!("{:#x}", self.get_dma_saddr()))
            .field("dma_daddr", &format_args!("{:#x}", self.dma_daddr))
            .field("dma_length", &self.dma_length)
            .field("dma_step_length", &self.dma_step_length)
            .field("dma_step_times", &self.dma_step_times)
            .field("dma_cmd", &self.dma_cmd)
            .finish()
    }
}

impl Debug for DMACmd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DMACmd")
            .field("cmd", &self.get_cmd())
            .field("rd_stat", &self.get_read_state())
            .field("wr_stat", &self.get_write_state())
            .field("trans_over", &self.get_trans_over())
            .field("single_trans_over", &self.get_single_trans_over())
            .field("int", &self.get_interrupt())
            .field("int_mask", &self.get_interrupt_mask())
            .finish()
    }
}

impl DMADesc {
    pub fn new(
        dma_order_addr: usize,
        dma_saddr: usize,
        dma_daddr: u32,
        buf_len: u32,
        dma_step_length: u32,
        dma_step_times: u32,
        dma_cmd: DMACmd,
    ) -> Self {
        let mut ret = Self {
            dma_order_addr_low: DMAOrderAddrLow::empty(),
            dma_saddr: 0,
            dma_daddr,
            dma_length: buf_len >> 2 + if buf_len & 3 != 0 { 1 } else { 0 },
            dma_step_length,
            dma_step_times,
            dma_cmd,
            reserved: 0,
            dma_order_addr_high: 0,
            dma_saddr_high: 0,
            _align: [0; 2],
        };
        ret.set_dma_order_addr(dma_order_addr, None);
        ret.set_dma_saddr(dma_saddr);

        ret
    }
    pub fn set_dma_order_addr(&mut self, addr: usize, ac97_mode: Option<(bool, bool, usize)>) {
        self.dma_order_addr_low.set_dma_addr(addr);
        if let Some((wr, stero, byte)) = ac97_mode {
            self.dma_order_addr_low.set_ac97_wr(wr);
            self.dma_order_addr_low.set_ac97_stero(stero);
            self.dma_order_addr_low.set_ac97_wr_byte_log(byte);
        }
        let u: usize = addr >> 32;
        self.dma_order_addr_high = u as u32;
    }
    pub fn get_dma_order_addr(&self) -> usize {
        (self.dma_order_addr_low.bits & 0xFFFF_FFF) as usize
            | ((self.dma_order_addr_high as usize) << 32)
    }
    pub fn set_dma_saddr(&mut self, saddr: usize) {
        self.dma_saddr = saddr as u32;
        let u: usize = saddr >> 32;
        self.dma_saddr_high = u as u32;
    }
    #[allow(unused)]
    pub fn set_dma_order_addr_enable(&mut self, stat: bool) {
        self.dma_order_addr_low.bits.set_bit(0, stat);
    }
    pub fn get_dma_saddr(&self) -> usize {
        ((self.dma_saddr_high as usize) << 32) | (self.dma_saddr as usize)
    }
}

impl_define_mem_reg_no_offset!(DMAOrder, DMA_ORDER, "芯片配置寄存器");
impl Debug for DMAOrder {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DMAOrder")
            .field("ask_addr", &format_args!("{:#x}", self.get_ask_addr()))
            .field("start", &self.is_dma_start())
            .field("stop", &self.is_dma_stop())
            .field("is_ask_valid", &self.is_ask_valid())
            .finish()
    }
}

#[allow(unused)]
impl DMAOrder {
    impl_get_set_field!(get_ask_addr, set_ask_addr, 5..=31);
    impl_get_set!(is_dma_stop, set_dma_stop, 4);
    impl_get_set!(is_dma_start, set_dma_start, 3);
    impl_get_set!(is_ask_valid, set_ask_valid, 2);
}
pub static mut DMA_DESC: DMADesc = DMADesc {
    dma_order_addr_low: DMAOrderAddrLow { bits: 0 },
    dma_saddr: 0,
    dma_daddr: 0,
    dma_length: 0,
    dma_step_length: 0,
    dma_step_times: 0,
    dma_cmd: DMACmd { bits: 0 },
    reserved: 0,
    dma_order_addr_high: 0,
    dma_saddr_high: 0,
    _align: [0; 2],
};
impl DMAOrder {
    #[allow(unused)]
    pub fn hard_clr(&self) {
        Self::empty().set_dma_stop(true).write();
        Self::empty().write();
    }
    pub fn wait() -> Result<(), ()> {
        loop {
            if !Self::read().is_dma_start() {
                break Ok(());
            }
            task::try_yield();
        }
    }
    #[inline(always)]
    pub fn dma_transfer(&mut self, dma_daddr: usize, buf: &[u8], is_send: bool) -> Result<(), ()> {
        assert!(buf.len() % 4 == 0);
        let dma_saddr: usize = &(buf[0]) as *const u8 as usize;
        let mut dma_cmd = DMACmd::empty();
        dma_cmd.set_send(is_send).set_interrupt_mask(true);
        debug!("[receive] {}", buf.len());
        unsafe {
            DMA_DESC = DMADesc::new(
                0,
                dma_saddr,
                (dma_daddr as u32) & 0x0FFF_FFFF,
                buf.len() as u32,
                0,
                1,
                dma_cmd,
            );
        }
        assert!(unsafe { addr_of!(DMA_DESC) } as usize % 32 == 0);
        let desc_addr_rsh5 = unsafe { (addr_of!(DMA_DESC) as usize) >> 5 };
        self.set_ask_addr(desc_addr_rsh5);
        self.set_dma_start(true);
        self.write();
        Self::wait()
    }
    pub fn send(&mut self, dma_daddr: usize, buf: &[u8]) -> Result<(), ()> {
        self.dma_transfer(dma_daddr, buf, true)
    }
    pub fn receive(&mut self, dma_daddr: usize, buf: &mut [u8]) -> Result<(), ()> {
        self.dma_transfer(dma_daddr, buf, false)
    }
}
