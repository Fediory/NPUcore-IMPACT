use super::BlockDevice;
use crate::arch::BLOCK_SZ;
use crate::arch::BUFFER_CACHE_NUM;
use crate::config::{MEMORY_HIGH_BASE, PAGE_SIZE, PAGE_SIZE_BITS};
use crate::mm::{frame_alloc, FrameTracker, KERNEL_SPACE};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

pub trait Cache {
    /// The read-only mapper to the block cache
    /// # Argument
    /// + `offset`: offset in cache
    /// + `f`: a closure to read
    fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V;
    /// The mutable mapper to the block cache
    /// # Argument
    /// + `offset`: offset in cache
    /// + `f`: a closure to write
    fn modify<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V;
    /// Tell cache to write back
    /// # Argument
    /// + `block_ids`: block ids in this cache
    /// + `block_device`: The pointer to the block_device.
    fn sync(&self, _block_ids: Vec<usize>, _block_device: &Arc<dyn BlockDevice>) {}
}

const PRIORITY_UPPERBOUND: usize = 1;
const BUFFER_SIZE: usize = BLOCK_SZ;
const PAGE_BUFFERS: usize = PAGE_SIZE / BUFFER_SIZE;

#[cfg(not(feature = "la64"))]
const BUFFER_CACHE_NUM: usize = 16;

const CACHEPOOLSIZE: usize = BUFFER_CACHE_NUM >> (BLOCK_SZ / 512).trailing_zeros();
const CACHEPOOLPAGE: usize = if (CACHEPOOLSIZE >> 3) > 1 {
    CACHEPOOLSIZE >> 3
} else {
    1
};

pub struct BufferCache {
    /// Every time kernel tried to alloc this buffer this number will increase 1(at most 3)
    /// When no free cache lefted this number will decrease 1(at least 0)
    /// When it's 0 and Arc's strong count is 1, this buffer will be writed back
    priority: usize,
    /// ***If block_id is usize::Max***, we assume it is an unused buffer.
    block_id: usize,
    dirty: bool,
    buffer: &'static mut [u8; BUFFER_SIZE],
}

impl Cache for BufferCache {
    fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        debug_assert!(offset.saturating_add(core::mem::size_of::<T>()) <= BUFFER_SIZE);
        f(unsafe {
            self.buffer
                .as_ptr()
                .add(offset)
                .cast::<T>()
                .as_ref()
                .unwrap()
        })
    }

    fn modify<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        self.dirty = true;
        debug_assert!(offset.saturating_add(core::mem::size_of::<T>()) <= BUFFER_SIZE);
        f(unsafe {
            self.buffer
                .as_mut_ptr()
                .add(offset)
                .cast::<T>()
                .as_mut()
                .unwrap()
        })
    }
}

impl BufferCache {
    pub fn new(buffer_ptr: *mut [u8; BUFFER_SIZE]) -> Self {
        let buffer = unsafe { buffer_ptr.as_mut().unwrap() };
        Self {
            priority: 0,
            block_id: usize::MAX,
            dirty: false,
            buffer,
        }
    }
    pub fn read_block(&mut self, block_id: usize, block_device: &Arc<dyn BlockDevice>) {
        self.block_id = block_id;
        let buf = self.buffer.as_mut();
        block_device.read_block(block_id, buf);
    }
}

pub struct BlockCacheManager {
    /// just hold all pages alloced
    _hold: Vec<Arc<FrameTracker>>,
    cache_pool: Vec<Arc<Mutex<BufferCache>>>,
}

impl BlockCacheManager {
    pub fn oom(&self, block_device: &Arc<dyn BlockDevice>) {
        for buffer_cache in &self.cache_pool {
            if Arc::strong_count(buffer_cache) > 1 {
                continue;
            }
            let mut locked = buffer_cache.lock();
            if locked.priority > 0 {
                locked.priority -= 1;
            } else {
                let block_id = locked.block_id;
                let buf = locked.buffer.as_ref();
                if locked.dirty {
                    block_device.write_block(block_id, buf);
                    locked.dirty = false;
                }
                locked.block_id = usize::MAX;
            }
        }
    }
    fn alloc_buffer_cache(&self, block_device: &Arc<dyn BlockDevice>) -> Arc<Mutex<BufferCache>> {
        loop {
            for buffer_cache in &self.cache_pool {
                let locked = buffer_cache.lock();
                if locked.block_id == usize::MAX {
                    return buffer_cache.clone();
                }
            }
            self.oom(block_device);
        }
    }
}

impl BlockCacheManager {
    pub const CACHE_SZ: usize = BUFFER_SIZE;

    pub fn new() -> Self {
        let mut hold: Vec<Arc<FrameTracker>> = Vec::new();
        let mut cache_pool: Vec<Arc<Mutex<BufferCache>>> = Vec::new();
        for i in 0..CACHEPOOLPAGE {
            hold.push(frame_alloc().unwrap());
            let page_ptr = (hold[i].ppn.0 << PAGE_SIZE_BITS) as *mut [u8; BUFFER_SIZE];
            for j in 0..PAGE_BUFFERS {
                let buffer_ptr = unsafe { page_ptr.add(j) };
                cache_pool.push(Arc::new(Mutex::new(BufferCache::new(buffer_ptr))))
            }
        }
        Self {
            _hold: hold,
            cache_pool,
        }
    }
    pub fn try_get_block_cache(&self, block_id: usize) -> Option<Arc<Mutex<BufferCache>>> {
        for buffer_cache in &self.cache_pool {
            let mut locked = buffer_cache.lock();
            if locked.block_id == block_id {
                if locked.priority < PRIORITY_UPPERBOUND {
                    locked.priority += 1;
                }
                return Some(buffer_cache.clone());
            }
        }
        None
    }

    pub fn get_block_cache(
        &self,
        block_id: usize,
        block_device: &Arc<dyn BlockDevice>,
    ) -> Arc<Mutex<BufferCache>> {
        match self.try_get_block_cache(block_id) {
            Some(block_cache) => block_cache,
            None => {
                let buffer_cache = self.alloc_buffer_cache(block_device);
                let mut locked = buffer_cache.lock();
                locked.read_block(block_id, block_device);
                if locked.priority < PRIORITY_UPPERBOUND {
                    locked.priority += 1;
                }
                drop(locked);
                buffer_cache
            }
        }
    }
}

/// PageCache is used for kernel.
/// Each PageCache contains PAGE_BUFFERS(8) BufferCache.
pub struct PageCache {
    /// Priority is used for out of memory
    /// Every time kernel tried to alloc this pagecache this number will increase 1(at most 1)
    /// Every time out of memory occurred this number will decrease 1(at least 0)
    /// When it's 0 and Arc's strong count is 1(one in inode) this PageCache will be dropped
    priority: usize,
    page_ptr: &'static mut [u8; PAGE_SIZE],
    tracker: Arc<FrameTracker>,
}

impl Cache for PageCache {
    fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        debug_assert!(offset.saturating_add(core::mem::size_of::<T>()) <= PAGE_SIZE);
        f(unsafe {
            self.page_ptr
                .as_ptr()
                .add(offset)
                .cast::<T>()
                .as_ref()
                .unwrap()
        })
    }

    fn modify<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        debug_assert!(offset.saturating_add(core::mem::size_of::<T>()) <= PAGE_SIZE);
        f(unsafe {
            self.page_ptr
                .as_mut_ptr()
                .add(offset)
                .cast::<T>()
                .as_mut()
                .unwrap()
        })
    }

    fn sync(&self, block_ids: Vec<usize>, block_device: &Arc<dyn BlockDevice>) {
        let lock = KERNEL_SPACE.try_lock();
        match lock {
            Some(lock) => {
                if !lock.is_dirty(self.tracker.ppn).unwrap() {
                    return;
                }
            }
            None => {}
        }
        self.write_back(block_ids, block_device)
    }
}

impl PageCache {
    pub fn new() -> Self {
        let tracker = unsafe { crate::mm::frame_alloc_uninit().unwrap() };
        let page_ptr = (tracker.ppn.0 << PAGE_SIZE_BITS) as *mut [u8; PAGE_SIZE];
        let page_ptr = unsafe { page_ptr.as_mut().unwrap() };
        Self {
            priority: 0,
            page_ptr,
            tracker,
        }
    }

    pub fn get_tracker(&self) -> Arc<FrameTracker> {
        self.tracker.clone()
    }
    pub fn read_in(&mut self, block_ids: Vec<usize>, block_device: &Arc<dyn BlockDevice>) {
        if block_ids.is_empty() {
            return;
        }
        assert!(block_ids.len() <= PAGE_BUFFERS);

        let mut start_block_id = usize::MAX;
        let mut con_length = 0;
        let mut start_buf_id = 0;
        for block_id in block_ids.iter() {
            if con_length == 0 {
                start_block_id = *block_id;
                con_length = 1;
            } else if *block_id != start_block_id + con_length {
                let buf = unsafe {
                    core::slice::from_raw_parts_mut(
                        self.page_ptr.as_mut_ptr().add(start_buf_id * BUFFER_SIZE),
                        con_length * BUFFER_SIZE,
                    )
                };
                block_device.read_block(start_block_id, buf);
                start_buf_id += con_length;
                start_block_id = *block_id;
                con_length = 1;
            } else {
                con_length += 1;
            }
        }
        let buf = unsafe {
            core::slice::from_raw_parts_mut(
                self.page_ptr.as_mut_ptr().add(start_buf_id * BUFFER_SIZE),
                con_length * BUFFER_SIZE,
            )
        };
        block_device.read_block(start_block_id, buf);
        self.page_ptr[block_ids.len() * BUFFER_SIZE..].fill(0);
        KERNEL_SPACE
            .lock()
            .clear_dirty_bit((self.tracker.ppn.0 | MEMORY_HIGH_BASE).into())
            .unwrap();
    }

    pub fn write_back(&self, block_ids: Vec<usize>, block_device: &Arc<dyn BlockDevice>) {
        if block_ids.is_empty() {
            return;
        }

        let mut start_block_id = usize::MAX;
        let mut con_length = 0;
        let mut start_buf_id = 0;
        for block_id in block_ids.iter() {
            if con_length == 0 {
                start_block_id = *block_id;
                con_length = 1;
            } else if *block_id != start_block_id + con_length {
                let buf = unsafe {
                    core::slice::from_raw_parts(
                        self.page_ptr.as_ptr().add(start_buf_id * BUFFER_SIZE),
                        con_length * BUFFER_SIZE,
                    )
                };
                block_device.write_block(start_block_id, buf);

                start_buf_id += con_length;
                start_block_id = *block_id;
                con_length = 1;
            } else {
                con_length += 1;
            }
        }
        let buf = unsafe {
            core::slice::from_raw_parts(
                self.page_ptr.as_ptr().add(start_buf_id * BUFFER_SIZE),
                con_length * BUFFER_SIZE,
            )
        };
        block_device.write_block(start_block_id, buf);
    }
}

pub struct PageCacheManager {
    cache_pool: Mutex<Vec<Option<Arc<Mutex<PageCache>>>>>,
    allocated_cache: Mutex<Vec<usize>>,
}

impl PageCacheManager {
    pub const CACHE_SZ: usize = PAGE_SIZE;

    pub fn new() -> Self {
        Self {
            cache_pool: Mutex::new(Vec::new()),
            allocated_cache: Mutex::new(Vec::new()),
        }
    }

    #[allow(unused)]
    pub fn try_get_cache(&self, inner_cache_id: usize) -> Option<Arc<Mutex<PageCache>>> {
        let lock = self.cache_pool.lock();
        if inner_cache_id >= lock.len() {
            return None;
        }
        let page_cache = lock[inner_cache_id].clone();
        if let Some(page_cache) = &page_cache {
            let mut locked = page_cache.lock();
            if locked.priority < PRIORITY_UPPERBOUND {
                locked.priority += 1;
            }
        }
        page_cache
    }

    pub fn get_cache<FUNC>(
        &self,
        inner_cache_id: usize,
        neighbor: FUNC,
        block_device: &Arc<dyn BlockDevice>,
    ) -> Arc<Mutex<PageCache>>
    where
        FUNC: Fn() -> Vec<usize>,
    {
        crate::mm::frame_reserve(1);
        let mut lock = self.cache_pool.lock();
        while inner_cache_id >= lock.len() {
            lock.push(None);
        }
        let page_cache = match &lock[inner_cache_id] {
            Some(page_cache) => page_cache.clone(),
            None => {
                let mut new_page_cache = PageCache::new();
                new_page_cache.read_in(neighbor(), &block_device);
                let new_page_cache = Arc::new(Mutex::new(new_page_cache));
                lock[inner_cache_id] = Some(new_page_cache.clone());
                self.allocated_cache.lock().push(inner_cache_id);
                new_page_cache
            }
        };
        let mut inner_lock = page_cache.lock();
        if inner_lock.priority < PRIORITY_UPPERBOUND {
            inner_lock.priority += 1;
        }
        drop(inner_lock);
        page_cache
    }

    pub fn oom<FUNC>(&self, neighbor: FUNC, block_device: &Arc<dyn BlockDevice>) -> usize
    where
        FUNC: Fn(usize) -> Vec<usize>,
    {
        let mut lock = self.cache_pool.lock();
        let mut dropped = 0;
        let mut new_allocated_cache = Vec::<usize>::new();

        for inner_cache_id in self.allocated_cache.lock().iter() {
            let inner_cache_id = *inner_cache_id;
            let inner = lock[inner_cache_id].as_ref().unwrap();
            if Arc::strong_count(inner) > 1 {
                new_allocated_cache.push(inner_cache_id);
                continue;
            }
            let mut inner_lock = inner.lock();
            if Arc::strong_count(&inner_lock.tracker) > 1 {
                new_allocated_cache.push(inner_cache_id);
            } else if inner_lock.priority > 0 {
                inner_lock.priority -= 1;
                new_allocated_cache.push(inner_cache_id);
            } else {
                let block_ids = neighbor(inner_cache_id);
                inner_lock.sync(block_ids, block_device);
                dropped += 1;
                drop(inner_lock);
                lock[inner_cache_id] = None;
            }
        }
        *self.allocated_cache.lock() = new_allocated_cache;
        dropped
    }

    pub fn notify_new_size(&self, new_size: usize) {
        let mut lock = self.cache_pool.lock();
        let new_pages = (new_size + PAGE_SIZE - 1) / PAGE_SIZE;
        while lock.len() > new_pages {
            lock.pop().unwrap().map(|cache| {
                if Arc::strong_count(&cache) > 1 {
                    panic!("page cache was used by others");
                }
            });
        }
        lock.shrink_to_fit();

        self.allocated_cache
            .lock()
            .retain(|cache_id| *cache_id < new_pages);
    }
}
