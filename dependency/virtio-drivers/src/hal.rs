use super::*; // 导入当前模块的父模块的所有 items

type VirtAddr = usize; // 定义类型别名 `VirtAddr` 表示虚拟地址
type PhysAddr = usize; // 定义类型别名 `PhysAddr` 表示物理地址

/// DMA 缓冲区
pub struct DMA {
    /// DMA 缓冲区为物理内存，`paddr` 表示 DMA 缓冲区的物理地址
    paddr: u32,
    /// DMA 缓冲区所占页面数
    pages: u32,
}

impl DMA {
    /// 创建一个新的 DMA 实例，`pages` 表示需要分配的页面数量
    pub fn new(pages: usize) -> Result<Self> {
        // 通过 `virtio_dma_alloc` 函数为 DMA 分配物理内存
        let paddr = unsafe { virtio_dma_alloc(pages) };
        if paddr == 0 {
            // 若分配内存失败，则返回错误
            return Err(Error::DmaError);
        }
        // 若分配内存成功，则返回一个 DMA 实例并记录该实例的物理地址和页面数
        Ok(DMA {
            paddr: paddr as u32, // 记录 DMA 缓冲区的物理地址
            pages: pages as u32, // 记录 DMA 缓冲区的页面数
        })
    }

    /// 获取 DMA 缓冲区的物理地址
    pub fn paddr(&self) -> usize {
        self.paddr as usize
    }

    /// 获取 DMA 缓冲区的虚拟地址
    pub fn vaddr(&self) -> usize {
        phys_to_virt(self.paddr as usize)
    }

    /// 获取 DMA 缓冲区对应页面的页框号
    pub fn pfn(&self) -> u32 {
        self.paddr >> 12 // 右移 12 位相当于除以 2 的 12 次方
    }

    /// 将 DMA 缓冲区的虚拟地址转换为一个可变的字节数组，并将其包装为一个指向静态生命周期的可扩展的引用
    pub unsafe fn as_buf(&self) -> &'static mut [u8] {
        core::slice::from_raw_parts_mut(self.vaddr() as _, PAGE_SIZE * self.pages as usize)
    }
}

impl Drop for DMA {
    /// 实现 `Drop` trait，当 DMA 对象被销毁时，通过 `virtio_dma_dealloc` 释放 DMA 缓冲区所占用的物理内存
    fn drop(&mut self) {
        let err = unsafe { virtio_dma_dealloc(self.paddr as usize, self.pages as usize) };
        assert_eq!(err, 0, "failed to deallocate DMA");
    }
}

/// 物理地址转虚拟地址
pub fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    unsafe { virtio_phys_to_virt(paddr) } // 调用 C 语言实现的 `virtio_phys_to_virt` 函数完成物理地址到虚拟地址的转换
}

/// 虚拟地址转物理地址
pub fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    unsafe { virtio_virt_to_phys(vaddr) } // 调用 C 语言实现的 `virtio_virt_to_phys` 函数完成虚拟地址到物理地址的转换
}

extern "C" {
    fn virtio_dma_alloc(pages: usize) -> PhysAddr;
    /// 使用物理地址释放 DMA 内存的函数
    fn virtio_dma_dealloc(paddr: PhysAddr, pages: usize) -> i32;
    /// 完成物理地址到虚拟地址的转换的函数
    fn virtio_phys_to_virt(paddr: PhysAddr) -> VirtAddr;
    /// 完成虚拟地址到物理地址的转换的函数
    fn virtio_virt_to_phys(vaddr: VirtAddr) -> PhysAddr;
}
