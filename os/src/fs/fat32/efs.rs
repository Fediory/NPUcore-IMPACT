#![allow(unused)]
use core::arch::asm;
use core::ptr::addr_of;

use crate::arch;

use super::{layout::BPB, Cache};
use super::{BlockCacheManager, BlockDevice, Fat};
use alloc::{sync::Arc, vec::Vec};

pub struct EasyFileSystem {
    /// Partition/Device the FAT32 is hosted on.
    pub block_device: Arc<dyn BlockDevice>,
    /// FAT information
    pub fat: Fat,
    /// The first data sector beyond the root directory
    pub data_area_start_block: u32,
    /// This is set to the cluster number of the first cluster of the root directory,
    /// usually 2 but not required to be 2.
    pub root_clus: u32,
    /// sector per cluster, usually 8 for SD card
    pub sec_per_clus: u8,
    /// Bytes per sector, 512 for SD card
    pub byts_per_sec: u16,
}

// export implementation of methods from FAT.
impl EasyFileSystem {
    #[inline(always)]
    pub fn this_fat_ent_offset(&self, n: u32) -> u32 {
        self.fat.this_fat_ent_offset(n) as u32
    }
    #[inline(always)]
    pub fn this_fat_sec_num(&self, n: u32) -> u32 {
        self.fat.this_fat_sec_num(n) as u32
    }
    #[inline(always)]
    pub fn get_next_clus_num(&self, result: u32) -> u32 {
        self.fat.get_next_clus_num(result, &self.block_device)
    }
}

impl EasyFileSystem {
    pub fn first_data_sector(&self) -> u32 {
        self.data_area_start_block
    }
    #[inline(always)]
    pub fn clus_size(&self) -> u32 {
        self.byts_per_sec as u32 * self.sec_per_clus as u32
    }
}

impl EasyFileSystem {
    /// For a given cluster number, calculate its first sector
    /// # Arguments
    /// + `clus_num`: cluster number
    /// # Return Value
    /// sector number
    #[inline(always)]
    pub fn first_sector_of_cluster(&self, clus_num: u32) -> u32 {
        debug_assert_eq!(self.sec_per_clus.count_ones(), 1);
        debug_assert!(clus_num >= 2);
        let start_block = self.data_area_start_block;
        let offset_blocks = (clus_num - 2) * self.sec_per_clus as u32;
        start_block + offset_blocks
    }
    /// Open the filesystem object.
    /// # Arguments
    /// + `block_device`: pointer of hardware device
    /// + `index_cache_mgr`: fat cache manager
    pub fn open(
        block_device: Arc<dyn BlockDevice>,
        index_cache_mgr: Arc<spin::Mutex<BlockCacheManager>>,
    ) -> Arc<Self> {
        // read SuperBlock
        let fat_cache_mgr = index_cache_mgr.clone();
        index_cache_mgr
            .lock()
            .get_block_cache(0, &block_device)
            .lock()
            .read(0, |super_block: &BPB| {
                // ***************Do NOT change this LINE!****************
                let byts_per_sec = super_block.byts_per_sec;
                debug_assert!(byts_per_sec as usize == arch::BLOCK_SZ);
                debug_assert!(BlockCacheManager::CACHE_SZ % byts_per_sec as usize == 0);
                debug_assert!(super_block.is_valid(), "Error loading EFS!");
                let efs = Self {
                    block_device,
                    fat: Fat::new(
                        super_block.rsvd_sec_cnt as usize,
                        byts_per_sec as usize,
                        (super_block.data_sector_count() / super_block.sec_per_clus as u32)
                            as usize,
                        fat_cache_mgr,
                    ),
                    root_clus: super_block.root_clus,
                    sec_per_clus: super_block.sec_per_clus,
                    byts_per_sec,
                    data_area_start_block: super_block.first_data_sector(),
                };
                Arc::new(efs)
            })
    }
    pub fn alloc_blocks(&self, blocks: usize) -> Vec<usize> {
        let sec_per_clus = self.sec_per_clus as usize;
        let alloc_num = (blocks - 1 + sec_per_clus) / sec_per_clus;
        let clus = self.fat.alloc(&self.block_device, alloc_num, None);
        debug_assert_eq!(clus.len(), alloc_num);
        let mut block_ids = Vec::<usize>::with_capacity(alloc_num * sec_per_clus);
        for clus_id in clus {
            let first_sec = self.first_sector_of_cluster(clus_id) as usize;
            for offset in 0..sec_per_clus {
                block_ids.push(first_sec + offset);
            }
        }
        block_ids
    }
}
