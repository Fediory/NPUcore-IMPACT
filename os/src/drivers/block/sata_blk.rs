use crate::config::PAGE_SIZE;
use crate::mm::{frame_alloc, frame_dealloc, PhysAddr};
use crate::drivers::block::BlockDevice;
use crate::arch::BLOCK_SZ;
use isomorphic_drivers::{
    provider,
    block::ahci::{AHCI, BLOCK_SIZE},
};
use log::info;
use spin::Mutex;
use pci::*;
pub struct SataBlock(Mutex<AHCI<Provider>>);

impl SataBlock {
    pub fn new() -> Self {
        Self(Mutex::new(pci_init().expect("AHCI new failed")))
    }
}

impl BlockDevice for SataBlock {
    fn read_block(&self, mut block_id: usize, buf: &mut [u8]) {
        // kernel BLOCK_SZ=2048, SATA BLOCK_SIZE=512，four times
        block_id = block_id * (BLOCK_SZ / BLOCK_SIZE);
        for buf in buf.chunks_mut(BLOCK_SIZE) {
            self.0
                .lock()
                .read_block(block_id as u64, buf);
            block_id += 1;
        }
    }

    fn write_block(&self, mut block_id: usize, buf: &[u8]) {
        block_id = block_id * (BLOCK_SZ / BLOCK_SIZE);
        for buf in buf.chunks(BLOCK_SIZE) {
            self.0
                .lock()
                .write_block(block_id as u64, buf);
            block_id += 1;
        }
    }
}

impl lwext4_rs::BlockDeviceInterface for SataBlock{
    fn read_block(&mut self, buf: &mut [u8], mut block_id: u64, block_count: u32) -> lwext4_rs::Result<usize> {
        // kernel BLOCK_SZ=2048, SATA BLOCK_SIZE=512，four times
        block_id = block_id * (BLOCK_SZ as u64 / BLOCK_SIZE as u64);
        for buf in buf.chunks_mut(BLOCK_SIZE) {
            self.0
                .lock()
                .read_block(block_id, buf);
            block_id += 1;
        }
        Ok(0)
    }

    fn write_block(&mut self, buf: &[u8], mut block_id: u64, block_count: u32) -> lwext4_rs::Result<usize> {
        block_id = block_id * (BLOCK_SZ as u64 / BLOCK_SIZE as u64);
        for buf in buf.chunks(BLOCK_SIZE) {
            self.0
                .lock()
                .write_block(block_id, buf);
            block_id += 1;
        }
        Ok(0)
    }
    
    fn close(&mut self) -> lwext4_rs::Result<()> {
        Ok(())
    }

    fn open(&mut self) -> lwext4_rs::Result<lwext4_rs::BlockDeviceConfig> {
        Ok(lwext4_rs::BlockDeviceConfig{
            block_size: BLOCK_SIZE as u32,
            block_count: 999,
            part_size: BLOCK_SIZE as u64 * 2,
            part_offset: 0
        })
    }

    fn lock(&mut self) -> lwext4_rs::Result<()> {
        Ok(())
    }

    fn unlock(&mut self) -> lwext4_rs::Result<()> {
        Ok(())
    }
}

pub struct Provider;

impl provider::Provider for Provider {
    const PAGE_SIZE: usize = PAGE_SIZE;
    fn alloc_dma(size: usize) -> (usize, usize) {
        let pages = size / PAGE_SIZE;
        let mut base = 0;
        for i in 0..pages {
            let frame = frame_alloc().unwrap();
            let frame_pa: PhysAddr = frame.ppn.into();
            let frame_pa = frame_pa.into();
            core::mem::forget(frame);
            if i == 0 {
                base = frame_pa;
            }
            assert_eq!(frame_pa, base + i * PAGE_SIZE);
        }
        let base_page = base / PAGE_SIZE;
        info!("virtio_dma_alloc: {:#x} {}", base_page, pages);
        (base, base)
    }

    fn dealloc_dma(va: usize, size: usize) {
        info!("dealloc_dma: {:x} {:x}", va, size);
        let pages = size / PAGE_SIZE;
        let mut pa = va;
        for _ in 0..pages {
            frame_dealloc(PhysAddr::from(pa).into());
            pa += PAGE_SIZE;
        }
    }
}


const PCI_CONFIG_ADDRESS: usize = 0xFE_0000_0000;
const PCI_COMMAND: u16 = 0x04;

struct UnusedPort;
impl PortOps for UnusedPort {
    unsafe fn read8(&self, _port: u16) -> u8 {0}
    unsafe fn read16(&self, _port: u16) -> u16 {0}
    unsafe fn read32(&self, _port: u16) -> u32 {0}
    unsafe fn write8(&self, _port: u16, _val: u8) {}
    unsafe fn write16(&self, _port: u16, _val: u16) {}
    unsafe fn write32(&self, _port: u16, _val: u32) {}
}

unsafe fn enable(loc: Location) {
    let ops = &UnusedPort;
    let am = CSpaceAccessMethod::MemoryMapped;

    let orig = am.read16(ops, loc, PCI_COMMAND);
    // bit0     |bit1       |bit2          |bit3           |bit10
    // IO Space |MEM Space  |Bus Mastering |Special Cycles |PCI Interrupt Disable
    am.write32(ops, loc, PCI_COMMAND, (orig | 0x40f) as u32);
    // Use PCI legacy interrupt instead
    // IO Space | MEM Space | Bus Mastering | Special Cycles
    am.write32(ops, loc, PCI_COMMAND, (orig | 0xf) as u32);
}

pub fn pci_init() -> Option<AHCI<Provider>> {
    for dev in unsafe {
        scan_bus(
            &UnusedPort,
            CSpaceAccessMethod::MemoryMapped,
            PCI_CONFIG_ADDRESS,
        )
    } {
        info!(
            "pci: {:02x}:{:02x}.{} {:#x} {:#x} ({} {}) irq: {}:{:?}",
            dev.loc.bus,
            dev.loc.device,
            dev.loc.function,
            dev.id.vendor_id,
            dev.id.device_id,
            dev.id.class,
            dev.id.subclass,
            dev.pic_interrupt_line,
            dev.interrupt_pin
        );
        dev.bars.iter().enumerate().for_each(|(index, bar)| {
            if let Some(BAR::Memory(pa, len, _, t)) = bar {
                info!("\tbar#{} (MMIO) {:#x} [{:#x}] [{:?}]", index, pa, len, t);
            } else if let Some(BAR::IO(pa, len)) = bar {
                info!("\tbar#{} (IO) {:#x} [{:#x}]", index, pa, len);
            }
        });
        if dev.id.class == 0x01 && dev.id.subclass == 0x06 {
            // Mass storage class, SATA subclass
            if let Some(BAR::Memory(pa, len, _, _)) = dev.bars[0] {
                if pa == 0 {
                    continue;
                }
                info!("Found AHCI device");
                if dev.status | Status::CAPABILITIES_LIST == Status::empty() {
                    info!("\tNo capabilities list");
                    return None;
                }
                unsafe { enable(dev.loc) };
                if let Some(x) = AHCI::new(pa as usize, len as usize) {
                    return Some(x);
                }
            }
        }
    }
    None
}