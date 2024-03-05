use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use spin::Mutex;

#[derive(Debug)]
pub enum ZramError {
    InvalidIndex,
    NoSpace,
    NotAllocated,
}

#[derive(Debug)]
pub struct ZramTracker(pub usize);

impl Drop for ZramTracker {
    fn drop(&mut self) {
        ZRAM_DEVICE.lock().discard(self.0).unwrap();
    }
}

pub struct Zram {
    compressed: Vec<Option<Vec<u8>>>,
    recycled: Vec<u16>,
    tail: u16,
}

impl Zram {
    pub fn new(capacity: usize) -> Self {
        let mut compressed = Vec::with_capacity(capacity);
        compressed.resize(compressed.capacity(), None);
        Self {
            compressed,
            recycled: Vec::new(),
            tail: 0,
        }
    }
    fn insert(&mut self, data: Vec<u8>) -> Result<Arc<ZramTracker>, ZramError> {
        let zram_id = match self.recycled.pop() {
            Some(zram_id) => zram_id as usize,
            None => {
                if self.tail as usize == self.compressed.len() {
                    return Err(ZramError::NoSpace);
                } else {
                    self.tail += 1;
                    (self.tail - 1) as usize
                }
            }
        };
        self.compressed[zram_id] = Some(data);
        Ok(Arc::new(ZramTracker(zram_id)))
    }
    fn get(&self, zram_id: usize) -> Result<&Vec<u8>, ZramError> {
        if zram_id >= self.compressed.len() {
            return Err(ZramError::InvalidIndex);
        }
        match &self.compressed[zram_id] {
            Some(compressed_data) => Ok(compressed_data),
            None => Err(ZramError::NotAllocated),
        }
    }
    fn remove(&mut self, zram_id: usize) -> Result<Vec<u8>, ZramError> {
        if zram_id >= self.compressed.len() {
            return Err(ZramError::InvalidIndex);
        }
        if zram_id == (self.tail - 1) as usize {
            self.tail = zram_id as u16;
        } else {
            self.recycled.push(zram_id as u16);
        }
        match self.compressed[zram_id].take() {
            Some(compressed_data) => Ok(compressed_data),
            None => Err(ZramError::NotAllocated),
        }
    }
    pub fn read(&mut self, zram_id: usize, buf: &mut [u8]) -> Result<(), ZramError> {
        match self.get(zram_id) {
            Ok(compressed_data) => {
                let decompressed_data =
                    decompress_size_prepended(compressed_data.as_slice()).unwrap();
                buf.copy_from_slice(decompressed_data.as_slice());
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
    pub fn write(&mut self, buf: &[u8]) -> Result<Arc<ZramTracker>, ZramError> {
        let mut compressed = compress_prepend_size(buf);
        compressed.shrink_to_fit();
        log::trace!("[zram] compressed len: {}", compressed.len());
        self.insert(compressed)
    }
    #[inline(always)]
    pub fn discard(&mut self, zram_id: usize) -> Result<(), ZramError> {
        match self.remove(zram_id) {
            Ok(_) => Ok(()),
            Err(error) => Err(error),
        }
    }
}

lazy_static! {
    pub static ref ZRAM_DEVICE: Arc<Mutex<Zram>> = Arc::new(Mutex::new(Zram::new(2048)));
}
