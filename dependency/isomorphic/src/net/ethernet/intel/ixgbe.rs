//! Intel 10Gb Network Adapter 82599 i.e. ixgbe network driver
//! Datasheet: https://www.intel.com/content/dam/www/public/us/en/documents/datasheets/82599-10-gbe-controller-datasheet.pdf

use super::super::structs::EthernetAddress;
use crate::provider::Provider;
use alloc::vec::Vec;
use bit_field::*;
use bitflags::*;
use core::hint::spin_loop;
use core::marker::PhantomData;
use core::mem::{size_of, MaybeUninit};
use core::slice;
use core::sync::atomic::{fence, Ordering};
use volatile::Volatile;

// On linux, use ip link set <dev> mtu <MTU - 18>
const IXGBE_MTU: usize = 8000;
const IXGBE_BUFFER_SIZE: usize = 8192;
const IXGBE_SEND_DESC_NUM: usize = 256;
const IXGBE_SEND_QUEUE_SIZE: usize = size_of::<IXGBESendDesc>() * IXGBE_SEND_DESC_NUM;
const IXGBE_SEND_QUEUE_NUM: usize = 4;
const IXGBE_RECV_DESC_NUM: usize = 256;
const IXGBE_RECV_QUEUE_SIZE: usize = size_of::<IXGBERecvDesc>() * IXGBE_RECV_DESC_NUM;

// At the beginning, all transmit descriptors have there status non-zero,
// so we need to track whether we are using the descriptor for the first time.
// When the descriptors wrap around, we set first_trans to false,
// and lookup status instead for checking whether it is empty.

pub struct IXGBE<P: Provider> {
    provider: PhantomData<P>,
    header: usize,
    size: usize,
    mac: EthernetAddress,
    registers: &'static mut [Volatile<u32>],
    send_queues: [&'static mut [IXGBESendDesc]; IXGBE_SEND_QUEUE_NUM],
    send_buffers: [[usize; IXGBE_SEND_DESC_NUM]; IXGBE_SEND_QUEUE_NUM],
    recv_queue: &'static mut [IXGBERecvDesc],
    recv_buffers: [usize; IXGBE_RECV_DESC_NUM],
    first_trans: [bool; IXGBE_SEND_QUEUE_NUM],
}

struct Send {
    descriptors: &'static mut [IXGBESendDesc],
    buffers: [usize; IXGBE_SEND_DESC_NUM],
    first_trans: bool,
}

struct Recv {
    descriptors: &'static mut [IXGBERecvDesc],
    buffers: [usize; IXGBE_RECV_DESC_NUM],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct IXGBESendDesc {
    addr: u64,
    len: u16,
    cso: u8,
    cmd: u8,
    status: u8,
    css: u8,
    vlan: u16,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct IXGBERecvDesc {
    addr: u64,
    len: u16,
    frag_chksum: u16,
    status_error: u16,
    vlan_tag: u16,
}

bitflags! {
    struct IXGBEStatus : u32 {
        // if LANID1 is clear, this is LAN0
        // if LANID1 is set, this is LAN1
        const LANID1 = 1 << 2;
        const LINK_UP = 1 << 7;
        const NUM_VFS1 = 1 << 10;
        const NUM_VFS2 = 1 << 11;
        const NUM_VFS4 = 1 << 12;
        const NUM_VFS8 = 1 << 13;
        const NUM_VFS16 = 1 << 14;
        const NUM_VFS32 = 1 << 15;
        const NUM_VFS64 = 1 << 16;
        const IOV = 1 << 18;
        const PCIE_MASTER_ENABLE = 1 << 19;
    }
}

// Device registers
const IXGBE_CTRL: usize = 0x00000 / 4;
const IXGBE_STATUS: usize = 0x00008 / 4;
const IXGBE_CTRL_EXT: usize = 0x00018 / 4;
const IXGBE_EICR: usize = 0x00800 / 4;
const IXGBE_EITR: usize = 0x00820 / 4;
const IXGBE_EIMS: usize = 0x00880 / 4;
const IXGBE_EIMC: usize = 0x00888 / 4;
const IXGBE_GPIE: usize = 0x00898 / 4;
const IXGBE_IVAR: usize = 0x00900 / 4;
const IXGBE_EIMC1: usize = 0x00AB0 / 4;
const IXGBE_EIMC2: usize = 0x00AB4 / 4;
const IXGBE_RDBAL: usize = 0x01000 / 4;
const IXGBE_RDBAH: usize = 0x01004 / 4;
const IXGBE_RDLEN: usize = 0x01008 / 4;
const IXGBE_DCA_RXCTRL: usize = 0x0100C / 4;
const IXGBE_RDH: usize = 0x01010 / 4;
const IXGBE_SRRCTL: usize = 0x01014 / 4;
const IXGBE_RDT: usize = 0x01018 / 4;
const IXGBE_RXDCTL: usize = 0x01028 / 4;
const IXGBE_RTRPT4C: usize = 0x02140 / 4;
const IXGBE_RTRPT4C_END: usize = 0x02160 / 4;
const IXGBE_RTRPCS: usize = 0x02430 / 4;
const IXGBE_RDRXCTL: usize = 0x02F00 / 4;
const IXGBE_PFQDE: usize = 0x02F04 / 4;
const IXGBE_RXCTRL: usize = 0x03000 / 4;
const IXGBE_RTRUP2TC: usize = 0x03020 / 4;
const IXGBE_FCTTV: usize = 0x03200 / 4;
const IXGBE_FCTTV_END: usize = 0x03210 / 4;
const IXGBE_FCRTL: usize = 0x03220 / 4;
const IXGBE_FCRTL_END: usize = 0x03240 / 4;
const IXGBE_FCRTH: usize = 0x03260 / 4;
const IXGBE_FCRTH_END: usize = 0x03280 / 4;
const IXGBE_FCRTV: usize = 0x032A0 / 4;
const IXGBE_RXPBSIZE: usize = 0x03C00 / 4;
const IXGBE_RXPBSIZE_END: usize = 0x03C20 / 4;
const IXGBE_FCCFG: usize = 0x03D00 / 4;
const IXGBE_HLREG0: usize = 0x04240 / 4;
const IXGBE_MAXFRS: usize = 0x04268 / 4;
const IXGBE_MFLCN: usize = 0x04294 / 4;
const IXGBE_AUTOC: usize = 0x042A0 / 4;
const IXGBE_LINKS: usize = 0x042A4 / 4;
const IXGBE_AUTOC2: usize = 0x04324 / 4;
const IXGBE_RTTDCS: usize = 0x04900 / 4;
const IXGBE_RTTDT1C: usize = 0x04908 / 4;
const IXGBE_RTTDT2C: usize = 0x04910 / 4;
const IXGBE_RTTDT2C_END: usize = 0x04930 / 4;
const IXGBE_RTTDQSEL: usize = 0x04904 / 4;
const IXGBE_RTTDC1C: usize = 0x04908 / 4;
const IXGBE_TXPBTHRESH: usize = 0x04950 / 4;
const IXGBE_TXPBTHRESH_END: usize = 0x04970 / 4;
const IXGBE_DMATXCTL: usize = 0x04A80 / 4;
const IXGBE_RXCSUM: usize = 0x05000 / 4;
const IXGBE_FCTRL: usize = 0x05080 / 4;
const IXGBE_PFVTCTL: usize = 0x051B0 / 4;
const IXGBE_MTA: usize = 0x05200 / 4;
const IXGBE_MTA_END: usize = 0x05400 / 4;
const IXGBE_MRQC: usize = 0x05818 / 4;
const IXGBE_TDBAL: usize = 0x06000 / 4;
const IXGBE_TDBAL_GAP: usize = 0x40 / 4;
const IXGBE_TDBAH: usize = 0x06004 / 4;
const IXGBE_TDBAH_GAP: usize = 0x40 / 4;
const IXGBE_TDLEN: usize = 0x06008 / 4;
const IXGBE_TDLEN_GAP: usize = 0x40 / 4;
const IXGBE_TDH: usize = 0x06010 / 4;
const IXGBE_TDH_GAP: usize = 0x40 / 4;
const IXGBE_TDT: usize = 0x06018 / 4;
const IXGBE_TDT_GAP: usize = 0x40 / 4;
const IXGBE_TXDCTL: usize = 0x06028 / 4;
const IXGBE_TXDCTL_GAP: usize = 0x40 / 4;
const IXGBE_DTXMXSZRQ: usize = 0x08100 / 4;
const IXGBE_MTQC: usize = 0x08120 / 4;
const IXGBE_SECRXCTRL: usize = 0x08D00 / 4;
const IXGBE_SECRXSTAT: usize = 0x08D04 / 4;
const IXGBE_VFTA: usize = 0x0A000 / 4;
const IXGBE_VFTA_END: usize = 0x0A200 / 4;
const IXGBE_RAL: usize = 0x0A200 / 4;
const IXGBE_RAH: usize = 0x0A204 / 4;
const IXGBE_MPSAR: usize = 0x0A600 / 4;
const IXGBE_MPSAR_END: usize = 0x0A800 / 4;
const IXGBE_RTTUP2TC: usize = 0x0C800 / 4;
const IXGBE_TXPBSIZE: usize = 0x0CC00 / 4;
const IXGBE_TXPBSIZE_END: usize = 0x0CC20 / 4;
const IXGBE_RTTPCS: usize = 0x0CD00 / 4;
const IXGBE_RTTPT2C: usize = 0x0CD20 / 4;
const IXGBE_RTTPT2C_END: usize = 0x0CD40 / 4;
const IXGBE_RTTPT2S: usize = 0x0CD40 / 4;
const IXGBE_RTTPT2S_END: usize = 0x0CD60 / 4;
const IXGBE_PFVLVF: usize = 0x0F100 / 4;
const IXGBE_PFVLVF_END: usize = 0x0F200 / 4;
const IXGBE_PFVLVFB: usize = 0x0F200 / 4;
const IXGBE_PFVLVFB_END: usize = 0x0F400 / 4;
const IXGBE_PFUTA: usize = 0x0F400 / 4;
const IXGBE_PFUTA_END: usize = 0x0F600 / 4;
const IXGBE_EEC: usize = 0x10010 / 4;

impl<P: Provider> IXGBE<P> {
    pub fn try_handle_interrupt(&mut self) -> bool {
        let ixgbe = &mut self.registers;

        let icr = ixgbe[IXGBE_EICR].read();
        if icr != 0 {
            // clear it
            ixgbe[IXGBE_EICR].write(icr);
            if icr.get_bit(20) {
                // link status change
                let status = ixgbe[IXGBE_LINKS].read();
                if status.get_bit(7) {
                    // link up
                    info!("ixgbe: interface link up");
                } else {
                    // link down
                    info!("ixgbe: interface link down");
                }
            }
            true
        } else {
            false
        }
    }

    pub fn get_mac(&self) -> EthernetAddress {
        self.mac
    }

    pub const fn get_mtu() -> usize {
        IXGBE_MTU
    }

    pub fn recv(&mut self) -> Option<Vec<u8>> {
        let ixgbe = &mut self.registers;

        let mut rdt = ixgbe[IXGBE_RDT].read();
        let index = (rdt as usize + 1) % IXGBE_RECV_DESC_NUM;
        let recv_desc = &mut self.recv_queue[index];
        if !recv_desc.status_error.get_bit(0) {
            return None;
        }
        let buffer = unsafe {
            slice::from_raw_parts(
                self.recv_buffers[index] as *const u8,
                recv_desc.len as usize,
            )
        };

        recv_desc.status_error.set_bit(0, false);

        rdt = (rdt + 1) % IXGBE_RECV_DESC_NUM as u32;
        ixgbe[IXGBE_RDT].write(rdt);
        Some(buffer.to_vec())
    }

    pub fn can_send(&mut self) -> bool {
        let ixgbe = &mut self.registers;

        for queue in 0..IXGBE_SEND_QUEUE_NUM {
            let tdt = ixgbe[IXGBE_TDT + IXGBE_TDT_GAP * queue].read();
            let index = (tdt as usize) % IXGBE_SEND_DESC_NUM;
            let send_desc = &mut self.send_queues[queue][index];
            if self.first_trans[queue] || send_desc.status.get_bit(0) {
                return true;
            }
        }
        false
    }

    pub fn send(&mut self, data: &[u8]) -> bool {
        self.msend(&[data]) > 0
    }

    pub fn msend(&mut self, data: &[&[u8]]) -> usize {
        let ixgbe = &mut self.registers;
        let mut data_index = 0;
        for queue in 0..IXGBE_SEND_QUEUE_NUM {
            let send_queue = &mut self.send_queues[queue];

            let mut tdt = ixgbe[IXGBE_TDT + IXGBE_TDT_GAP * queue].read();

            while data_index < data.len() {
                let index = (tdt as usize) % IXGBE_SEND_DESC_NUM;
                let send_desc = &mut send_queue[index];

                if !(self.first_trans[queue] || send_desc.status.get_bit(0)) {
                    break;
                }
                let len = data[data_index].len();
                let target = unsafe {
                    slice::from_raw_parts_mut(self.send_buffers[queue][index] as *mut u8, len)
                };
                target.copy_from_slice(data[data_index]);
                send_desc.len = len as u16;
                // RS | IFCS | EOP
                send_desc.cmd = (1 << 3) | (1 << 1) | (1 << 0);
                send_desc.status = 0;

                tdt = (tdt + 1) % IXGBE_SEND_DESC_NUM as u32;

                // round
                if tdt == 0 {
                    self.first_trans[queue] = false;
                }
                data_index += 1;
                fence(Ordering::SeqCst);
                ixgbe[IXGBE_TDT + IXGBE_TDT_GAP * queue].write(tdt);
            }
        }
        data_index
    }

    pub fn new(header: usize, size: usize) -> Self {
        assert_eq!(size_of::<IXGBESendDesc>(), 16);
        assert_eq!(size_of::<IXGBERecvDesc>(), 16);

        info!("ixgbe: interface setup begin");

        let (recv_queue_va, recv_queue_pa) = P::alloc_dma(IXGBE_RECV_QUEUE_SIZE);
        let mut recv_queue = unsafe {
            slice::from_raw_parts_mut(recv_queue_va as *mut IXGBERecvDesc, IXGBE_RECV_DESC_NUM)
        };
        let mut recv_buffers = [0; IXGBE_RECV_DESC_NUM];

        let ixgbe = unsafe { slice::from_raw_parts_mut(header as *mut Volatile<u32>, size / 4) };
        debug!(
            "status before setup: {:#?}",
            IXGBEStatus::from_bits_truncate(ixgbe[IXGBE_STATUS].read())
        );

        // 4.6.3 Initialization Sequence

        // 4.6.3.1 Interrupts During Initialization
        // 1. Disable interrupts.
        // mask all interrupts
        ixgbe[IXGBE_EIMC].write(!0);
        ixgbe[IXGBE_EIMC1].write(!0);
        ixgbe[IXGBE_EIMC2].write(!0);

        // 2. Issue a global reset.
        // reset: LRST | RST
        ixgbe[IXGBE_CTRL].write(1 << 3 | 1 << 26);
        while ixgbe[IXGBE_CTRL].read() & (1 << 3 | 1 << 26) != 0 {}

        // 3. Disable interrupts (again).
        // mask all interrupts
        ixgbe[IXGBE_EIMC].write(!0);
        ixgbe[IXGBE_EIMC1].write(!0);
        ixgbe[IXGBE_EIMC2].write(!0);

        // 4.6.3.2 Global Reset and General Configuration
        // no flow control
        for reg in (IXGBE_FCTTV..IXGBE_FCTTV_END).step_by(4) {
            ixgbe[reg].write(0);
        }
        for reg in (IXGBE_FCRTL..IXGBE_FCRTL_END).step_by(4) {
            ixgbe[reg].write(0);
        }
        for reg in (IXGBE_FCRTH..IXGBE_FCRTH_END).step_by(4) {
            ixgbe[reg].write(0);
        }
        ixgbe[IXGBE_FCRTV].write(0);
        ixgbe[IXGBE_FCCFG].write(0);

        // Auto-Read Done
        while !ixgbe[IXGBE_EEC].read().get_bit(9) {
            spin_loop();
        }

        // DMA Init Done
        while !ixgbe[IXGBE_RDRXCTL].read().get_bit(3) {
            spin_loop();
        }

        // BAM, Accept Broadcast packets
        ixgbe[IXGBE_FCTRL].write(ixgbe[IXGBE_FCTRL].read() | (1 << 10));

        // 4.6.7 Receive Initialization

        // Receive Address (RAL[n] and RAH[n]) for used addresses.
        // Read MAC Address
        let ral = ixgbe[IXGBE_RAL].read();
        let rah = ixgbe[IXGBE_RAH].read();
        let mac: [u8; 6] = [
            ral as u8,
            (ral >> 8) as u8,
            (ral >> 16) as u8,
            (ral >> 24) as u8,
            rah as u8,
            (rah >> 8) as u8,
        ];
        debug!("mac {:x?}", mac);

        // Unicast Table Array (PFUTA).
        for i in IXGBE_PFUTA..IXGBE_PFUTA_END {
            ixgbe[i].write(0);
        }
        // VLAN Filter Table Array (VFTA[n]).
        for i in IXGBE_VFTA..IXGBE_VFTA_END {
            ixgbe[i].write(0);
        }
        // VLAN Pool Filter (PFVLVF[n]).
        for i in IXGBE_PFVLVF..IXGBE_PFVLVF_END {
            ixgbe[i].write(0);
        }
        // MAC Pool Select Array (MPSAR[n]).
        for i in IXGBE_MPSAR..IXGBE_MPSAR_END {
            ixgbe[i].write(0);
        }
        // VLAN Pool Filter Bitmap (PFVLVFB[n]).
        for i in IXGBE_PFVLVFB..IXGBE_PFVLVFB_END {
            ixgbe[i].write(0);
        }
        // Set up the Multicast Table Array (MTA) registers. This entire table should be zeroed and only the desired multicast addresses should be permitted (by writing 0x1 to the corresponding bit location).
        for i in IXGBE_MTA..IXGBE_MTA_END {
            ixgbe[i].write(0);
        }

        // Program the different Rx filters and Rx offloads via registers FCTRL, VLNCTRL, MCSTCTRL, RXCSUM, RQTC, RFCTL, MPSAR, RSSRK, RETA, SAQF, DAQF, SDPQF, FTQF, SYNQF, ETQF, ETQS, RDRXCTL, RSCDBU.
        // IPPCSE IP Payload Checksum Enable
        ixgbe[IXGBE_RXCSUM].update(|x| {
            x.set_bit(12, true);
        });

        // Note that RDRXCTL.CRCStrip and HLREG0.RXCRCSTRP must be set to the same value. At the same time the RDRXCTL.RSCFRSTSIZE should be set to 0x0 as opposed to its hardware default.
        // CRCStrip | RSCACKC | FCOE_WRFIX
        ixgbe[IXGBE_RDRXCTL].update(|x| {
            *x |= (1 << 0) | (1 << 25) | (1 << 26);
        });

        // Setup receive queue 0..
        // The following steps should be done once per transmit queue:
        // 2. Receive buffers of appropriate size should be allocated and pointers to these buffers should be stored in the descriptor ring.
        for i in 0..IXGBE_RECV_DESC_NUM {
            let (buffer_page_va, buffer_page_pa) = P::alloc_dma(IXGBE_BUFFER_SIZE);
            recv_queue[i].addr = buffer_page_pa as u64;
            recv_buffers[i] = buffer_page_va;
        }

        // 3. Program the descriptor base address with the address of the region (registers RDBAL, RDBAH).
        ixgbe[IXGBE_RDBAL].write(recv_queue_pa as u32); // RDBAL
        ixgbe[IXGBE_RDBAH].write((recv_queue_pa >> 32) as u32); // RDBAH

        // 4. Set the length register to the size of the descriptor ring (register RDLEN).
        ixgbe[IXGBE_RDLEN].write(IXGBE_RECV_QUEUE_SIZE as u32); // RDLEN

        // 5. Program SRRCTL associated with this queue according to the size of the buffers and the required header control.
        // Legacy descriptor, 8K buffer size
        // Enable packet drop
        ixgbe[IXGBE_SRRCTL].update(|x| {
            *x = (*x & !0xf) | (8 << 0) | (1 << 28);
        });

        ixgbe[IXGBE_RDH].write(0); // RDH

        // 8. Program RXDCTL with appropriate values including the queue Enable bit. Note that packets directed to a disabled queue are dropped.
        ixgbe[IXGBE_RXDCTL].update(|x| {
            x.set_bit(25, true);
        }); // enable queue

        // 9. Poll the RXDCTL register until the Enable bit is set. The tail should not be bumped before this bit was read as 1b.
        while !ixgbe[IXGBE_RXDCTL].read().get_bit(25) {
            spin_loop(); // wait for it
        }

        // 10. Bump the tail pointer (RDT) to enable descriptors fetching by setting it to the ring length minus one.
        ixgbe[IXGBE_RDT].write((IXGBE_RECV_DESC_NUM - 1) as u32); // RDT

        // all queues are setup
        // 11. Enable the receive path by setting RXCTRL.RXEN. This should be done only after all other settings are done following the steps below.
        // Halt the receive data path by setting SECRXCTRL.RX_DIS bit.
        ixgbe[IXGBE_SECRXCTRL].update(|x| {
            x.set_bit(1, true);
        });
        // Wait for the data paths to be emptied by HW. Poll the SECRXSTAT.SECRX_RDY bit until it is asserted by HW.
        while !ixgbe[IXGBE_SECRXSTAT].read().get_bit(0) {
            spin_loop(); // poll
        }

        // Set RXCTRL.RXEN
        // enable the queue
        ixgbe[IXGBE_RXCTRL].update(|x| {
            x.set_bit(0, true);
        });
        // Clear the SECRXCTRL.SECRX_DIS bits to enable receive data path
        ixgbe[IXGBE_SECRXCTRL].update(|x| {
            x.set_bit(1, false);
        });

        // Set bit 16 of the CTRL_EXT register and clear bit 12 of the DCA_RXCTRL[n] register[n].
        ixgbe[IXGBE_CTRL_EXT].update(|x| {
            x.set_bit(16, true);
        });
        ixgbe[IXGBE_DCA_RXCTRL].update(|x| {
            x.set_bit(12, false);
        });

        // 4.6.8 Transmit Initialization

        // Program the HLREG0 register according to the required MAC behavior.
        // TXCRCEN | RXCRCSTRP | JUMBOEN | TXPADEN | RXLNGTHERREN
        ixgbe[IXGBE_HLREG0].update(|x| {
            *x |= (1 << 0) | (1 << 1) | (1 << 2) | (1 << 10) | (1 << 27);
        });
        // Set max MTU
        ixgbe[IXGBE_MAXFRS].write((IXGBE_MTU as u32) << 16);

        // The following steps should be done once per transmit queue:
        let mut send_queues: [&'static mut [IXGBESendDesc]; IXGBE_SEND_QUEUE_NUM] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let mut send_buffers = [[0; IXGBE_SEND_DESC_NUM]; IXGBE_SEND_QUEUE_NUM];
        for queue in 0..IXGBE_SEND_QUEUE_NUM {
            let (send_queue_va, send_queue_pa) = P::alloc_dma(IXGBE_SEND_QUEUE_SIZE);
            let send_queue = unsafe {
                slice::from_raw_parts_mut(send_queue_va as *mut IXGBESendDesc, IXGBE_SEND_DESC_NUM)
            };
            // 1. Allocate a region of memory for the transmit descriptor list.
            for i in 0..IXGBE_SEND_DESC_NUM {
                let (buffer_page_va, buffer_page_pa) = P::alloc_dma(IXGBE_BUFFER_SIZE);
                send_queue[i].addr = buffer_page_pa as u64;
                send_buffers[queue][i] = buffer_page_va;
            }
            send_queues[queue] = send_queue;

            // 2. Program the descriptor base address with the address of the region (TDBAL, TDBAH).
            ixgbe[IXGBE_TDBAL + IXGBE_TDBAL_GAP * queue].write(send_queue_pa as u32); // TDBAL
            ixgbe[IXGBE_TDBAH + IXGBE_TDBAH_GAP * queue].write((send_queue_pa >> 32) as u32); // TDBAH

            // 3. Set the length register to the size of the descriptor ring (TDLEN).
            ixgbe[IXGBE_TDLEN + IXGBE_TDLEN_GAP * queue].write(IXGBE_SEND_QUEUE_SIZE as u32); // TDLEN
            ixgbe[IXGBE_TDH + IXGBE_TDH_GAP * queue].write(0); // TDH
            ixgbe[IXGBE_TDT + IXGBE_TDT_GAP * queue].write(0); // TDT

            if queue == 0 {
                // 6. Enable transmit path by setting DMATXCTL.TE. This step should be executed only for the first enabled transmit queue and does not need to be repeated for any following queues.
                ixgbe[IXGBE_DMATXCTL].update(|x| {
                    x.set_bit(0, true);
                });
            }

            // 7. Enable the queue using TXDCTL.ENABLE. Poll the TXDCTL register until the Enable bit is set.
            ixgbe[IXGBE_TXDCTL + IXGBE_TXDCTL_GAP * queue].update(|x| {
                x.set_bit(25, true);
            });
            while !ixgbe[IXGBE_TXDCTL + IXGBE_TXDCTL_GAP * queue]
                .read()
                .get_bit(25)
            {
                spin_loop();
            }
        }

        // 4.6.6 Interrupt Initialization
        // The software driver associates between Tx and Rx interrupt causes and the EICR register by setting the IVAR[n] registers.
        // map Rx0 to interrupt 0
        ixgbe[IXGBE_IVAR].write(0b00000000_00000000_00000000_10000000);
        // Set the interrupt throttling in EITR[n] and GPIE according to the preferred mode of operation.
        // Throttle interrupts
        // Seems having good effect on tx bandwidth
        // but bad effect on rx bandwidth
        // CNT_WDIS | ITR Interval=100us
        // if sys_read() spin more times, the interval here should be larger
        // Linux use dynamic ETIR based on statistics
        ixgbe[IXGBE_EITR].write(((100 / 2) << 3) | (1 << 31));
        // Disable general purpose interrupt
        // We don't need them
        ixgbe[IXGBE_GPIE].write(0);
        // Software clears EICR by writing all ones to clear old interrupt causes.
        // clear all interrupt
        ixgbe[IXGBE_EICR].write(!0);

        debug!(
            "status after setup: {:#?}",
            IXGBEStatus::from_bits_truncate(ixgbe[IXGBE_STATUS].read())
        );

        info!("ixgbe: interface setup done");

        IXGBE {
            provider: PhantomData,
            header,
            size,
            mac: EthernetAddress::from_bytes(&mac),
            registers: ixgbe,
            send_queues,
            send_buffers,
            recv_queue,
            recv_buffers,
            first_trans: [true; IXGBE_SEND_QUEUE_NUM],
        }
    }

    pub fn enable_irq(&mut self) {
        // Software enables the required interrupt causes by setting the EIMS register.
        // unmask rx interrupt and link status change
        self.registers[IXGBE_EIMS].write((1 << 0) | (1 << 20));
    }

    pub fn disable_irq(&mut self) {
        // Software enables the required interrupt causes by setting the EIMS register.
        // mask all interrupts
        self.registers[IXGBE_EIMS].write(0);
    }

    pub fn is_link_up(&self) -> bool {
        let status = self.registers[IXGBE_LINKS].read();
        status.get_bit(7)
    }
}

impl<P: Provider> Drop for IXGBE<P> {
    fn drop(&mut self) {
        let ixgbe = &mut self.registers;
        // 1. Disable interrupts.
        ixgbe[IXGBE_EIMC].write(!0);
        ixgbe[IXGBE_EIMC1].write(!0);
        ixgbe[IXGBE_EIMC2].write(!0);

        // 2. Issue a global reset.
        // reset: LRST | RST
        ixgbe[IXGBE_CTRL].write(1 << 3 | 1 << 26);
        while ixgbe[IXGBE_CTRL].read() & (1 << 3 | 1 << 26) != 0 {
            spin_loop();
        }
        for send_queue in self.send_queues.iter() {
            P::dealloc_dma(send_queue.as_ptr() as usize, IXGBE_SEND_QUEUE_SIZE);
        }
        P::dealloc_dma(self.recv_queue.as_ptr() as usize, IXGBE_RECV_QUEUE_SIZE);
        for buffer in self.send_buffers.iter() {
            for &send_buffer in buffer.iter() {
                P::dealloc_dma(send_buffer, IXGBE_BUFFER_SIZE);
            }
        }
        for &recv_buffer in self.recv_buffers.iter() {
            P::dealloc_dma(recv_buffer, IXGBE_BUFFER_SIZE);
        }

        info!("ixgbe: interface detached");
    }
}
