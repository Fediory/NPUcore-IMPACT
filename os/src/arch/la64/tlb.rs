use core::arch::asm;

#[inline(always)]
/// Invalidate non-global TLB entries
pub fn tlb_invalidate() {
    unsafe {
        asm!("invtlb 0x3,$zero, $zero");
    }
}
#[inline(always)]
pub fn tlb_global_invalidate() {
    unsafe {
        asm!("invtlb 0x0,$zero, $zero");
    }
}



// use super::{ASId, TLBEHi, TLBIdx, TLBEL, TLBELO0, TLBELO1};
// use crate::mm::{PhysPageNum, VirtPageNum};

// pub const USR_ASID: usize = 0;
// pub const KERN_ASID: usize = (1 << 10) - 1;
// #[inline(always)]
// /// Set Adress Space ID of current core to the low 10 bits of `asid`
// pub fn set_asid(asid: usize) {
//     let mut id = ASId::read();
//     id.set_asid(asid & (1 << id.get_asid_width() - 1)).write();
// }
// pub fn tlb_addr_allow_write(vpn: VirtPageNum, ppn: PhysPageNum) -> Result<(), ()> {
//     TLBEHi::read().set_vppn(vpn).write();
//     tlbsrch();
//     let ret = TLBIdx::read();
//     if ret.is_non_existent() {
//         return Err(());
//     } else {
//         if vpn.0 & 1 == 0 {
//             TLBELO0::read().set_ppn(ppn).set_dirty(true).write();
//         } else {
//             TLBELO1::read().set_ppn(ppn).set_dirty(true).write();
//         }
//         Ok(())
//     }
// }

// pub fn tlb_read(idx: usize) -> Result<(PhysPageNum, PhysPageNum), ()> {
//     TLBIdx::read().set_index(idx).write();
//     let ret = TLBIdx::read();

//     tlbrd();

//     if ret.is_non_existent() {
//         Err(())
//     } else {
//         Ok((TLBELO0::read().get_ppn(), TLBELO1::read().get_ppn()))
//     }
// }
// pub fn tlb_search(vpn: VirtPageNum) -> Result<PhysPageNum, ()> {
//     TLBEHi::read().set_vppn(vpn).write();

//     tlbsrch();

//     let ret = TLBIdx::read();
//     if ret.is_non_existent() {
//         Err(())
//     } else {
//         if vpn.0 & 1 == 0 {
//             Ok(tlb_read(ret.get_index()).unwrap().0)
//         } else {
//             Ok(tlb_read(ret.get_index()).unwrap().1)
//         }
//     }
// }

// fn tlbrd() {
//     unsafe {
//         asm!("tlbrd");
//     }
// }
// fn tlbsrch() {
//     unsafe {
//         asm!("tlbsrch");
//     }
// }
