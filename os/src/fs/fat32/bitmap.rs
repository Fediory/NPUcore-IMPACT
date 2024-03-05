use super::layout::BAD_BLOCK;
use super::{BlockCacheManager, BlockDevice, Cache};
use alloc::{collections::VecDeque, sync::Arc, vec::Vec};
use spin::{Mutex, MutexGuard};

const VACANT_CLUS_CACHE_SIZE: usize = 64;
const FAT_ENTRY_FREE: u32 = 0;
const FAT_ENTRY_RESERVED_TO_END: u32 = 0x0FFF_FFF8;
pub const EOC: u32 = 0x0FFF_FFFF;
/// *In-memory* data structure
/// In FAT32, there are 2 FATs by default. We use ONLY the first one.
pub struct Fat {
    /// Cache manager for fat
    fat_cache_mgr: Arc<Mutex<BlockCacheManager>>,
    /// The first block id of FAT.
    /// In FAT32, this is equal to bpb.rsvd_sec_cnt
    start_block_id: usize,
    /// size fo sector in bytes copied from BPB
    byts_per_sec: usize,
    /// The total number of FAT entries
    tot_ent: usize,
    /// The queue used to store known vacant clusters
    vacant_clus: Mutex<VecDeque<u32>>,
    /// The final unused cluster id we found
    hint: Mutex<usize>,
}

impl Fat {
    /// Get the next cluster number pointed by current fat entry.
    /// # Arguments
    /// + `current_clus_num`: current cluster number
    /// + `block_device`: pointer of block device
    /// # Return value
    /// Next cluster number
    pub fn get_next_clus_num(
        &self,
        current_clus_num: u32,
        block_device: &Arc<dyn BlockDevice>,
    ) -> u32 {
        self.fat_cache_mgr
            .lock()
            .get_block_cache(
                self.this_fat_sec_num(current_clus_num) as usize,
                block_device,
            )
            .lock()
            .read(
                self.this_fat_ent_offset(current_clus_num) as usize,
                |fat_entry: &u32| -> u32 { *fat_entry },
            )
            & EOC
    }
    /// Get all cluster numbers after the current cluster number
    /// # Arguments
    /// + `current_clus_num`: current cluster number
    /// + `block_device`: pointer of block device
    /// # Return value
    /// List of cluster numbers
    pub fn get_all_clus_num(
        &self,
        mut current_clus_num: u32,
        block_device: &Arc<dyn BlockDevice>,
    ) -> Vec<u32> {
        let mut v = Vec::with_capacity(8);
        loop {
            v.push(current_clus_num);
            current_clus_num = self.get_next_clus_num(current_clus_num, &block_device);
            if [BAD_BLOCK, FAT_ENTRY_FREE].contains(&current_clus_num)
                || current_clus_num >= FAT_ENTRY_RESERVED_TO_END
            {
                break;
            }
        }
        v
    }

    /// Constructor for fat
    /// # Argument
    /// + `rsvd_sec_cnt`: size in bytes of BPB
    /// + `byts_per_sec`: bytes per sector
    /// + `clus`: the total numebr of FAT entries
    /// + `fat_cache_mgr`: fat cache manager
    /// # Return value
    /// Fat
    pub fn new(
        rsvd_sec_cnt: usize,
        byts_per_sec: usize,
        clus: usize,
        fat_cache_mgr: Arc<Mutex<BlockCacheManager>>,
    ) -> Self {
        Self {
            //used_marker: Default::default(),
            fat_cache_mgr,
            start_block_id: rsvd_sec_cnt,
            byts_per_sec,
            tot_ent: clus,
            vacant_clus: spin::Mutex::new(VecDeque::new()),
            hint: Mutex::new(0),
        }
    }

    /// For a given cluster number, calculate its sector ID in the fat region
    /// # Argument
    /// + `clus_num`: cluster number
    /// # Return value
    /// sector ID
    #[inline(always)]
    pub fn this_fat_sec_num(&self, clus_num: u32) -> usize {
        let fat_offset = clus_num * 4;
        (self.start_block_id as u32 + (fat_offset / (self.byts_per_sec as u32))) as usize
    }
    #[inline(always)]
    /// For a given cluster number, calculate its offset in the sector of the fat region
    /// # Argument
    /// + `clus_num`: cluster number
    /// # Return value
    /// offset
    pub fn this_fat_ent_offset(&self, clus_num: u32) -> usize {
        let fat_offset = clus_num * 4;
        (fat_offset % (self.byts_per_sec as u32)) as usize
    }
    /// Assign the cluster entry to `current` to `next`
    /// If `current` is None, ignore this operation
    /// # Argument
    /// + `block_device`: pointer of block device
    /// + `current`: current cluster number
    /// + `next`: next cluster to set
    fn set_next_clus(&self, block_device: &Arc<dyn BlockDevice>, current: Option<u32>, next: u32) {
        if current.is_none() {
            return;
        }
        let current = current.unwrap();
        self.fat_cache_mgr
            .lock()
            .get_block_cache(self.this_fat_sec_num(current) as usize, block_device)
            .lock()
            .modify(
                self.this_fat_ent_offset(current as u32),
                |bitmap_block: &mut u32| {
                    //println!("[set_next_clus]bitmap_block:{}->{}", *bitmap_block, next);
                    *bitmap_block = next;
                },
            )
    }

    /// Allocate as many clusters (but not greater than alloc_num) as possible.
    /// # Argument
    /// + `block_device`: The target block_device.
    /// + `alloc_num`: The number of clusters to allocate.
    /// + `last`: The preceding cluster of the one to be allocated.
    /// # Return value
    /// List of cluster numbers
    pub fn alloc(
        &self,
        block_device: &Arc<dyn BlockDevice>,
        alloc_num: usize,
        mut last: Option<u32>,
    ) -> Vec<u32> {
        let mut allocated_cluster = Vec::with_capacity(alloc_num);
        // A lock is required to guarantee mutual exclusion between processes.
        let mut hlock = self.hint.lock();
        for _ in 0..alloc_num {
            last = self.alloc_one(block_device, last, &mut hlock);
            if last.is_none() {
                // There is no more free cluster.
                // Or `last` next cluster is valid.
                log::error!("[alloc]: alloc error, last: {:?}", last);
                break;
            }
            allocated_cluster.push(last.unwrap());
        }
        self.set_next_clus(block_device, last, EOC);
        allocated_cluster
    }

    /// Find and allocate an cluster from data area.
    /// # Argument
    /// + `block_device`: The target block_device.
    /// + `last`: The preceding cluster of the one to be allocated.
    /// + `hlock`: The lock of hint(Fat).
    /// # Return value
    /// If successful, return allocated cluster number
    /// otherwise, return None
    fn alloc_one(
        &self,
        block_device: &Arc<dyn BlockDevice>,
        last: Option<u32>,
        hlock: &mut MutexGuard<usize>,
    ) -> Option<u32> {
        if last.is_some() {
            let next_cluster_of_current = self.get_next_clus_num(last.unwrap(), block_device);
            debug_assert!(next_cluster_of_current >= FAT_ENTRY_RESERVED_TO_END);
        }
        // Now we can allocate clusters freely

        // Get a free cluster from `vacant_clus`
        if let Some(free_clus_id) = self.vacant_clus.lock().pop_back() {
            self.set_next_clus(block_device, last, free_clus_id);
            return Some(free_clus_id);
        }

        // Allocate a free cluster starts with `hint`
        let start = **hlock;
        let free_clus_id = self.get_next_free_clus(start as u32, block_device);
        if free_clus_id.is_none() {
            return None;
        }
        let free_clus_id = free_clus_id.unwrap();
        **hlock = (free_clus_id + 1) as usize % self.tot_ent;

        self.set_next_clus(block_device, last, free_clus_id);
        Some(free_clus_id)
    }

    /// Find next free cluster from data area.
    /// # Argument
    /// + `start`: The cluster id to traverse to find the next free cluster
    /// + `block_device`: The target block_device.
    /// # Return value
    /// If successful, return free cluster number
    /// otherwise, return None
    fn get_next_free_clus(&self, start: u32, block_device: &Arc<dyn BlockDevice>) -> Option<u32> {
        for clus_id in start..self.tot_ent as u32 {
            if FAT_ENTRY_FREE == self.get_next_clus_num(clus_id, block_device) {
                return Some(clus_id);
            }
        }
        for clus_id in 0..start {
            if FAT_ENTRY_FREE == self.get_next_clus_num(clus_id, block_device) {
                return Some(clus_id);
            }
        }
        None
    }

    /// Free multiple clusters from the data area.
    /// # Argument
    /// + `block_device`: Pointer to block_device.
    /// + `cluster_list`: List of clusters that need to be freed
    pub fn free(
        &self,
        block_device: &Arc<dyn BlockDevice>,
        cluster_list: Vec<u32>,
        last: Option<u32>,
    ) {
        // Before freeing, a lock
        let mut lock = self.vacant_clus.lock();
        for cluster_id in cluster_list {
            self.set_next_clus(block_device, Some(cluster_id), FAT_ENTRY_FREE);
            if lock.len() < VACANT_CLUS_CACHE_SIZE {
                lock.push_back(cluster_id);
            }
        }
        self.set_next_clus(block_device, last, EOC);
    }
}
