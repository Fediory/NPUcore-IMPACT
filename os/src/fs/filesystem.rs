use core::ops::AddAssign;

use alloc::sync::Arc;
use lazy_static::*;
use spin::Mutex;

pub enum FS {
    Null,
    Fat32,
}

pub struct FileSystem {
    pub fs_id: usize,
    pub fs_type: FS,
}

lazy_static! {
    static ref FS_ID_COUNTER: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
}

impl FileSystem {
    pub fn new(fs_type: FS) -> Self {
        FS_ID_COUNTER.lock().add_assign(1);
        let fs_id = *FS_ID_COUNTER.lock();
        Self { fs_id, fs_type }
    }
}
