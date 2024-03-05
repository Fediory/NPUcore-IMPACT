use super::BlockDevice;
use crate::mm::{
    frame_alloc, frame_dealloc, kernel_token, FrameTracker, PageTable, PageTableImpl, PhysAddr,
    PhysPageNum, StepByOne, VirtAddr,
};
use alloc::{sync::Arc, vec::Vec};
use lazy_static::*;
use spin::Mutex;
use virtio_drivers::{VirtIOBlk, VirtIOHeader};
const VIRT_IO_BLOCK_SZ: usize = 512;
#[allow(unused)]
const VIRTIO0: usize = 0x1000_1000;

pub struct VirtIOBlock(Mutex<VirtIOBlk<'static>>);

lazy_static! {
    static ref QUEUE_FRAMES: Mutex<Vec<Arc<FrameTracker>>> = Mutex::new(Vec::new());
}

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, mut block_id: usize, buf: &mut [u8]) {
        for buf in buf.chunks_mut(VIRT_IO_BLOCK_SZ) {
            self.0
                .lock()
                .read_block(block_id, buf)
                .expect("Error when reading VirtIOBlk");
            block_id += 1;
        }
    }
    fn write_block(&self, mut block_id: usize, buf: &[u8]) {
        for buf in buf.chunks(VIRT_IO_BLOCK_SZ) {
            self.0
                .lock()
                .write_block(block_id, buf)
                .expect("Error when writing VirtIOBlk");
            block_id += 1;
        }
    }
}

impl VirtIOBlock {
    #[allow(unused)]
    pub fn new() -> Self {
        Self(Mutex::new(
            VirtIOBlk::new(unsafe { &mut *(VIRTIO0 as *mut VirtIOHeader) }).unwrap(),
        ))
    }
}

#[no_mangle]
pub extern "C" fn virtio_dma_alloc(pages: usize) -> PhysAddr {
    let mut ppn_base = PhysPageNum(0);
    for i in 0..pages {
        let frame = frame_alloc().unwrap();
        if i == 0 {
            ppn_base = frame.ppn;
        }
        assert_eq!(frame.ppn.0, ppn_base.0 + i);
        QUEUE_FRAMES.lock().push(frame);
    }
    ppn_base.into()
}

#[no_mangle]
pub extern "C" fn virtio_dma_dealloc(pa: PhysAddr, pages: usize) -> i32 {
    let mut ppn_base: PhysPageNum = pa.into();
    for _ in 0..pages {
        frame_dealloc(ppn_base);
        ppn_base.step();
    }
    0
}

#[no_mangle]
pub extern "C" fn virtio_phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    VirtAddr(paddr.0)
}

lazy_static! {
    static ref KERNEL_TOKEN: usize = kernel_token();
}

#[no_mangle]
pub extern "C" fn virtio_virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    PageTableImpl::from_token(*KERNEL_TOKEN)
        .translate_va(vaddr)
        .unwrap()
}
