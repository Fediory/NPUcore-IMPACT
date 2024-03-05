#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main() -> i32 {
    println!("\x1b[1m\x1b[93mHello! It's NPUCore\x1b[0m");
    println!("\x1b[1m\x1b[34mSchool\x1b[0m: Northwestern Polytechnical University");
    println!("\x1b[1m\x1b[34mDeveloper\x1b[0m: Zhaoxiang Huang, Yuxuan Lin, Sundi Guan");
    0
}