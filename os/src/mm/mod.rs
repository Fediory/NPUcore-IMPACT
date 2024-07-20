pub mod address;
mod frame_allocator;
mod heap_allocator;
mod map_area;
mod memory_set;
mod page_table;
#[cfg(feature = "zram")]
mod zram;
use core::alloc::Layout;

pub use crate::arch::KernelPageTableImpl;
pub use crate::arch::PageTableImpl;
use address::VPNRange;
pub use address::{PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
pub use frame_allocator::{
    frame_alloc, frame_alloc_uninit, frame_dealloc, frame_reserve, unallocated_frames, FrameTracker,
};
use heap_allocator::HEAP_ALLOCATOR;
pub use map_area::{Frame, MapFlags, MapPermission};
pub use memory_set::{MemoryError, MemorySet, KERNEL_SPACE};
pub use page_table::{
    copy_from_user, copy_from_user_array, copy_to_user, copy_to_user_array, copy_to_user_debug,
    copy_to_user_string, get_from_user, translated_byte_buffer,
    translated_byte_buffer_append_to_existing_vec, translated_ref, translated_refmut,
    translated_str, try_get_from_user, PageTable, UserBuffer,
};

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.lock().activate();
}
pub use crate::arch::tlb_invalidate;

#[macro_export]
/// Convert user pointer trg to `Some(*trg)` or `None` if null.
macro_rules! move_ptr_to_opt {
    ($trg:ident) => {
        if $trg != null() {
            let t = *translated_ref(current_user_token(), $trg);
            Some(t)
        } else {
            None
        }
    };
    ($token:ident,$trg:ident) => {
        if $trg != null() {
            let t = *translated_ref($token, $trg);
            Some(t)
        } else {
            None
        }
    };
}

#[macro_export]
/// Convert user pointer `trg:*const T` to `Some(trg as & T)` or `None` if null.
macro_rules! ptr_to_opt_ref {
    ($trg:ident) => {
        if $trg != null() {
            Some(translated_ref(current_user_token(), $trg))
        } else {
            None
        }
    };
    ($token:ident,$trg:ident) => {
        if $trg != null() {
            Some(translated_ref($token, $trg))
        } else {
            None
        }
    };
}

#[no_mangle]
pub extern "C" fn ext4_user_malloc(size: ::core::ffi::c_size_t) -> *mut ::core::ffi::c_void {
    HEAP_ALLOCATOR
        .lock()
        .alloc(Layout::array::<u8>(size).unwrap())
        .unwrap()
        .as_ptr() as *mut ::core::ffi::c_void
}
