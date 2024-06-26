//! A set of drivers that should be work in both OS and user space.

#![feature(const_fn_trait_bound)]
#![no_std]
#![allow(unused_variables, dead_code)]

extern crate alloc;
#[cfg(feature = "log")]
#[macro_use]
extern crate log;

#[macro_use]
mod logging;

pub mod block;
pub mod net;
pub mod provider;
