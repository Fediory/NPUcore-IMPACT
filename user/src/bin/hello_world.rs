#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main() -> i32 {
    println!("\x1b[1m\x1b[93mHello! This is NPUCore-IMPACT!!!\x1b[0m");
    println!("\x1b[1m\x1b[34mSchool\x1b[0m: Northwestern Polytechnical University");
    println!("\x1b[1m\x1b[34mDeveloper\x1b[0m: Yixu Feng, Yifei Zhang, Hanchen Zhang");
    0
}
