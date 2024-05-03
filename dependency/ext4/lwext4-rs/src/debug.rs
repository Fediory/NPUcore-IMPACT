use crate::types::DebugFlags;
use lwext4_sys::ext4::*;

/// Set debug flags
pub fn set_debug_mask(debug_flags: DebugFlags) {
    unsafe { ext4_dmask_set(debug_flags.bits()) }
}

/// Clear debug flags
pub fn clear_debug_mask(debug_flags: DebugFlags) {
    unsafe { ext4_dmask_clr(debug_flags.bits()) }
}

/// Get debug flags
pub fn get_debug_mask() -> DebugFlags {
    let mask = unsafe { ext4_dmask_get() };
    DebugFlags::from_bits_truncate(mask)
}
