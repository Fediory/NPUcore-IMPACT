extern crate rlibc;
use rlibc::memcmp;
#[no_mangle]
pub unsafe extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    memcmp(s1, s2, n)
}
#[no_mangle]
pub extern "C" fn _Unwind_Resume() {}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
