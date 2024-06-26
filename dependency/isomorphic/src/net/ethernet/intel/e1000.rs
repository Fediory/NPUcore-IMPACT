use alloc::slice;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::mem::size_of;
use core::sync::atomic::{fence, Ordering};

use bit_field::*;
use bitflags::*;
use volatile::Volatile;

use crate::net::ethernet::structs::EthernetAddress;
use crate::provider::Provider;

// At the beginning, all transmit descriptors have there status non-zero,
// so we need to track whether we are using the descriptor for the first time.
// When the descriptors wrap around, we set first_trans to false,
// and lookup status instead for checking whether it is empty.

pub struct E1000<P: Provider> {
    header: usize,
    size: usize,
    mac: EthernetAddress,
    provider: PhantomData<P>,
    registers: &'static mut [Volatile<u32>],
    send_queue: &'static mut [E1000SendDesc],
    send_buffers: Vec<usize>,
    recv_queue: &'static mut [E1000RecvDesc],
    recv_buffers: Vec<usize>,
    first_trans: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct E1000SendDesc {
    addr: u64,
    len: u16,
    cso: u8,
    cmd: u8,
    status: u8,
    css: u8,
    special: u8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct E1000RecvDesc {
    addr: u64,
    len: u16,
    chksum: u16,
    status: u16,
    error: u8,
    special: u8,
}

bitflags! {
    struct E1000Status : u32 {
        const FD = 1 << 0;
        const LU = 1 << 1;
        const TXOFF = 1 << 4;
        const TBIMODE = 1 << 5;
        const SPEED_100M = 1 << 6;
        const SPEED_1000M = 1 << 7;
        const ASDV_100M = 1 << 8;
        const ASDV_1000M = 1 << 9;
        const MTXCKOK = 1 << 10;
        const PCI66 = 1 << 11;
        const BUS64 = 1 << 12;
        const PCIX_MODE = 1 << 13;
        const GIO_MASTER_ENABLE = 1 << 19;
    }
}

impl<P: Provider> E1000<P> {
    pub fn new(header: usize, size: usize, mac: EthernetAddress) -> Self {
        assert_eq!(size_of::<E1000SendDesc>(), 16);
        assert_eq!(size_of::<E1000RecvDesc>(), 16);

        let (send_queue_va, send_queue_pa) = P::alloc_dma(P::PAGE_SIZE);
        let (recv_queue_va, recv_queue_pa) = P::alloc_dma(P::PAGE_SIZE);
        let send_queue = unsafe {
            slice::from_raw_parts_mut(
                send_queue_va as *mut E1000SendDesc,
                P::PAGE_SIZE / size_of::<E1000SendDesc>(),
            )
        };
        let recv_queue = unsafe {
            slice::from_raw_parts_mut(
                recv_queue_va as *mut E1000RecvDesc,
                P::PAGE_SIZE / size_of::<E1000RecvDesc>(),
            )
        };

        let mut send_buffers = Vec::with_capacity(send_queue.len());
        let mut recv_buffers = Vec::with_capacity(recv_queue.len());

        let e1000 = unsafe { slice::from_raw_parts_mut(header as *mut Volatile<u32>, size / 4) };
        debug!(
            "status before setup: {:#?}",
            E1000Status::from_bits_truncate(e1000[E1000_STATUS].read())
        );

        // 4.6 Software Initialization Sequence

        // 4.6.6 Transmit Initialization

        // Program the descriptor base address with the address of the region.
        e1000[E1000_TDBAL].write(send_queue_pa as u32); // TDBAL
        e1000[E1000_TDBAH].write((send_queue_pa >> 32) as u32); // TDBAH

        // Set the length register to the size of the descriptor ring.
        e1000[E1000_TDLEN].write(P::PAGE_SIZE as u32); // TDLEN

        // If needed, program the head and tail registers.
        e1000[E1000_TDH].write(0); // TDH
        e1000[E1000_TDT].write(0); // TDT

        for i in 0..send_queue.len() {
            let (buffer_page_va, buffer_page_pa) = P::alloc_dma(P::PAGE_SIZE);
            send_queue[i].addr = buffer_page_pa as u64;
            send_buffers.push(buffer_page_va);
        }

        // EN | PSP | CT=0x10 | COLD=0x40
        e1000[E1000_TCTL].write((1 << 1) | (1 << 3) | (0x10 << 4) | (0x40 << 12)); // TCTL
                                                                                   // IPGT=0xa | IPGR1=0x8 | IPGR2=0xc
        e1000[E1000_TIPG].write(0xa | (0x8 << 10) | (0xc << 20)); // TIPG

        // 4.6.5 Receive Initialization
        let mut ral: u32 = 0;
        let mut rah: u32 = 0;
        for i in 0..4 {
            ral = ral | (mac.as_bytes()[i] as u32) << (i * 8);
        }
        for i in 0..2 {
            rah = rah | (mac.as_bytes()[i + 4] as u32) << (i * 8);
        }

        e1000[E1000_RAL].write(ral); // RAL
                                     // AV | AS=DA
        e1000[E1000_RAH].write(rah | (1 << 31)); // RAH

        // MTA
        for i in E1000_MTA..E1000_RAL {
            e1000[i].write(0);
        }

        // Program the descriptor base address with the address of the region.
        e1000[E1000_RDBAL].write(recv_queue_pa as u32); // RDBAL
        e1000[E1000_RDBAH].write((recv_queue_pa >> 32) as u32); // RDBAH

        // Set the length register to the size of the descriptor ring.
        e1000[E1000_RDLEN].write(P::PAGE_SIZE as u32); // RDLEN

        // If needed, program the head and tail registers. Note: the head and tail pointers are initialized (by hardware) to zero after a power-on or a software-initiated device reset.
        e1000[E1000_RDH].write(0); // RDH

        // The tail pointer should be set to point one descriptor beyond the end.
        e1000[E1000_RDT].write((recv_queue.len() - 1) as u32); // RDT

        // Receive buffers of appropriate size should be allocated and pointers to these buffers should be stored in the descriptor ring.
        for i in 0..recv_queue.len() {
            let (buffer_page_va, buffer_page_pa) = P::alloc_dma(P::PAGE_SIZE);
            recv_queue[i].addr = buffer_page_pa as u64;
            recv_buffers.push(buffer_page_va);
        }

        // EN | BAM | BSIZE=3 | BSEX | SECRC
        // BSIZE=3 | BSEX means buffer size = 4096
        e1000[E1000_RCTL].write((1 << 1) | (1 << 15) | (3 << 16) | (1 << 25) | (1 << 26)); // RCTL

        debug!(
            "status after setup: {:#?}",
            E1000Status::from_bits_truncate(e1000[E1000_STATUS].read())
        );

        // enable interrupt
        // clear interrupt
        e1000[E1000_ICR].write(e1000[E1000_ICR].read());
        // RXT0
        e1000[E1000_IMS].write(1 << 7); // IMS

        // clear interrupt
        e1000[E1000_ICR].write(e1000[E1000_ICR].read());

        E1000 {
            header,
            size,
            mac,
            provider: PhantomData,
            registers: e1000,
            send_queue,
            send_buffers,
            recv_queue,
            recv_buffers,
            first_trans: true,
        }
    }

    pub fn handle_interrupt(&mut self) -> bool {
        let icr = self.registers[E1000_ICR].read();
        if icr != 0 {
            // clear it
            self.registers[E1000_ICR].write(icr);
            true
        } else {
            false
        }
    }

    pub fn receive(&mut self) -> Option<Vec<u8>> {
        let tdt = self.registers[E1000_TDT].read() as usize;
        let index = tdt % self.send_queue.len();
        let send_desc = &mut self.send_queue[index];

        let mut rdt = self.registers[E1000_RDT].read() as usize;
        let index = (rdt + 1) % self.recv_queue.len();
        let recv_desc = &mut self.recv_queue[index];

        let transmit_avail = self.first_trans || send_desc.status.get_bit(0);
        let receive_avail = recv_desc.status.get_bit(0);

        if !(transmit_avail && receive_avail) {
            return None;
        }
        let buffer = unsafe {
            slice::from_raw_parts(
                self.recv_buffers[index] as *const u8,
                recv_desc.len as usize,
            )
        };

        recv_desc.status.set_bit(0, false);

        rdt = index;
        self.registers[E1000_RDT].write(rdt as u32);

        Some(buffer.to_vec())
    }

    pub fn can_send(&self) -> bool {
        let tdt = self.registers[E1000_TDT].read();
        let index = (tdt as usize) % self.send_queue.len();
        let send_desc = &self.send_queue[index];
        self.first_trans || send_desc.status.get_bit(0)
    }

    pub fn send(&mut self, buffer: &[u8]) {
        let mut tdt = self.registers[E1000_TDT].read();
        let index = (tdt as usize) % self.send_queue.len();
        let send_desc = &mut self.send_queue[index];
        assert!(self.first_trans || send_desc.status.get_bit(0));

        let target =
            unsafe { slice::from_raw_parts_mut(self.send_buffers[index] as *mut u8, buffer.len()) };
        target.copy_from_slice(&buffer);

        send_desc.len = buffer.len() as u16 + 4;
        send_desc.cmd = (1 << 3) | (1 << 1) | (1 << 0); // RS | IFCS | EOP
        send_desc.status = 0;
        fence(Ordering::SeqCst);

        tdt = (tdt + 1) % self.send_queue.len() as u32;
        self.registers[E1000_TDT].write(tdt);
        fence(Ordering::SeqCst);

        // round
        if tdt == 0 {
            self.first_trans = false;
        }
    }
}

impl<P: Provider> Drop for E1000<P> {
    fn drop(&mut self) {
        P::dealloc_dma(self.send_queue.as_ptr() as usize, P::PAGE_SIZE);
        P::dealloc_dma(self.recv_queue.as_ptr() as usize, P::PAGE_SIZE);
        for &send_buffer in self.send_buffers.iter() {
            P::dealloc_dma(send_buffer, P::PAGE_SIZE);
        }
        for &recv_buffer in self.recv_buffers.iter() {
            P::dealloc_dma(recv_buffer, P::PAGE_SIZE);
        }
    }
}

const E1000_STATUS: usize = 0x0008 / 4;
const E1000_ICR: usize = 0x00C0 / 4;
const E1000_IMS: usize = 0x00D0 / 4;
const E1000_IMC: usize = 0x00D8 / 4;
const E1000_RCTL: usize = 0x0100 / 4;
const E1000_TCTL: usize = 0x0400 / 4;
const E1000_TIPG: usize = 0x0410 / 4;
const E1000_RDBAL: usize = 0x2800 / 4;
const E1000_RDBAH: usize = 0x2804 / 4;
const E1000_RDLEN: usize = 0x2808 / 4;
const E1000_RDH: usize = 0x2810 / 4;
const E1000_RDT: usize = 0x2818 / 4;
const E1000_TDBAL: usize = 0x3800 / 4;
const E1000_TDBAH: usize = 0x3804 / 4;
const E1000_TDLEN: usize = 0x3808 / 4;
const E1000_TDH: usize = 0x3810 / 4;
const E1000_TDT: usize = 0x3818 / 4;
const E1000_MTA: usize = 0x5200 / 4;
const E1000_RAL: usize = 0x5400 / 4;
const E1000_RAH: usize = 0x5404 / 4;
