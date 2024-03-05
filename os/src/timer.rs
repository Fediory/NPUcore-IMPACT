#![allow(unused)]
use core::cmp::Ordering;
use core::ops::{Add, AddAssign, Sub};

use crate::arch::get_clock_freq;
pub use crate::arch::get_time;

pub const MSEC_PER_SEC: usize = 1000;

pub const USEC_PER_SEC: usize = 1_000_000;
pub const USEC_PER_MSEC: usize = 1_000;

pub const NSEC_PER_SEC: usize = 1_000_000_000;
pub const NSEC_PER_MSEC: usize = 1_000_000;
pub const NSEC_PER_USEC: usize = 1_000;

/// Return current time measured by seconds.
pub fn get_time_sec() -> usize {
    let i = get_time() / (get_clock_freq());
    //log::info!("[timer.rs] get_time(): {},sec: {}", get_time(), i);
    i
}
/// Return current time measured by ms.
pub fn get_time_ms() -> usize {
    let i = get_time() / (get_clock_freq() / MSEC_PER_SEC);
    //log::info!("[timer.rs] get_time(): {},ms: {}", get_time(), i);
    i
}
/// Return current time measured by us.
pub fn get_time_us() -> usize {
    let i = get_time() / (get_clock_freq() / USEC_PER_SEC);
    //log::info!("[timer.rs] get_time(): {},us: {}", get_time(), i);
    i
}
/// Return current time measured by nano seconds.
pub fn get_time_ns() -> usize {
    let i = get_time() * NSEC_PER_SEC / (get_clock_freq());
    //log::info!("[timer.rs] get_time(): {},ns: {}", get_time(), i);
    i
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Traditional UNIX timespec structures represent elapsed time, measured by the system clock
/// # *CAUTION*
/// tv_sec & tv_usec should be usize.
#[repr(C)]
pub struct TimeSpec {
    /// The tv_sec member represents the elapsed time, in whole seconds.
    pub tv_sec: usize,
    /// The tv_usec member captures rest of the elapsed time, represented as the number of microseconds.
    pub tv_nsec: usize,
}
impl AddAssign for TimeSpec {
    fn add_assign(&mut self, rhs: Self) {
        self.tv_sec += rhs.tv_sec;
        self.tv_nsec += rhs.tv_nsec;
    }
}
impl Add for TimeSpec {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut sec = self.tv_sec + other.tv_sec;
        let mut nsec = self.tv_nsec + other.tv_nsec;
        sec += nsec / NSEC_PER_SEC;
        nsec %= NSEC_PER_SEC;
        Self {
            tv_sec: sec,
            tv_nsec: nsec,
        }
    }
}

impl Sub for TimeSpec {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let self_ns = self.to_ns();
        let other_ns = other.to_ns();
        if self_ns <= other_ns {
            TimeSpec::new()
        } else {
            TimeSpec::from_ns(self_ns - other_ns)
        }
    }
}

impl Ord for TimeSpec {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.tv_sec.cmp(&other.tv_sec) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.tv_nsec.cmp(&other.tv_nsec),
            Ordering::Greater => Ordering::Greater,
        }
    }
}

impl PartialOrd for TimeSpec {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl TimeSpec {
    pub fn new() -> Self {
        Self {
            tv_sec: 0,
            tv_nsec: 0,
        }
    }
    pub fn from_tick(tick: usize) -> Self {
        Self {
            tv_sec: tick / get_clock_freq(),
            tv_nsec: (tick % get_clock_freq()) * NSEC_PER_SEC / get_clock_freq(),
        }
    }
    pub fn from_s(s: usize) -> Self {
        Self {
            tv_sec: s,
            tv_nsec: 0,
        }
    }
    pub fn from_ms(ms: usize) -> Self {
        Self {
            tv_sec: ms / MSEC_PER_SEC,
            tv_nsec: (ms % MSEC_PER_SEC) * NSEC_PER_MSEC,
        }
    }
    pub fn from_us(us: usize) -> Self {
        Self {
            tv_sec: us / USEC_PER_SEC,
            tv_nsec: (us % USEC_PER_SEC) * NSEC_PER_USEC,
        }
    }
    pub fn from_ns(ns: usize) -> Self {
        Self {
            tv_sec: ns / NSEC_PER_SEC,
            tv_nsec: ns % NSEC_PER_SEC,
        }
    }
    pub fn to_ns(&self) -> usize {
        self.tv_sec * NSEC_PER_SEC + self.tv_nsec
    }
    pub fn is_zero(&self) -> bool {
        self.tv_sec == 0 && self.tv_nsec == 0
    }
    pub fn now() -> Self {
        TimeSpec::from_tick(get_time())
    }
}

/// Traditional UNIX timeval structures represent elapsed time, measured by the system clock
/// # *CAUTION*
/// tv_sec & tv_usec should be usize.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct TimeVal {
    /// The `tv_sec` member represents the elapsed time, in whole seconds
    pub tv_sec: usize,
    /// The `tv_nsec` member represents the rest of the elapsed time in nanoseconds.
    pub tv_usec: usize,
}

impl TimeVal {
    pub fn new() -> Self {
        Self {
            tv_sec: 0,
            tv_usec: 0,
        }
    }
    pub fn from_tick(tick: usize) -> Self {
        Self {
            tv_sec: tick / get_clock_freq(),
            tv_usec: (tick % get_clock_freq()) * USEC_PER_SEC / get_clock_freq(),
        }
    }
    pub fn to_tick(&self) -> usize {
        self.tv_sec * get_clock_freq() + self.tv_usec * get_clock_freq() / USEC_PER_SEC
    }
    pub fn from_s(s: usize) -> Self {
        Self {
            tv_sec: s,
            tv_usec: 0,
        }
    }
    pub fn from_ms(ms: usize) -> Self {
        Self {
            tv_sec: ms / MSEC_PER_SEC,
            tv_usec: (ms % MSEC_PER_SEC) * USEC_PER_MSEC,
        }
    }
    pub fn from_us(us: usize) -> Self {
        Self {
            tv_sec: us / USEC_PER_SEC,
            tv_usec: us % USEC_PER_SEC,
        }
    }
    pub fn to_us(&self) -> usize {
        self.tv_sec * USEC_PER_SEC + self.tv_usec
    }
    pub fn is_zero(&self) -> bool {
        self.tv_sec == 0 && self.tv_usec == 0
    }
    pub fn now() -> Self {
        TimeVal::from_tick(get_time())
    }
}

impl Add for TimeVal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut sec = self.tv_sec + other.tv_sec;
        let mut usec = self.tv_usec + other.tv_usec;
        sec += usec / USEC_PER_SEC;
        usec %= USEC_PER_SEC;
        Self {
            tv_sec: sec,
            tv_usec: usec,
        }
    }
}

impl Sub for TimeVal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let self_us = self.to_us();
        let other_us = other.to_us();
        if self_us <= other_us {
            TimeVal::new()
        } else {
            TimeVal::from_us(self_us - other_us)
        }
    }
}

impl Ord for TimeVal {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.tv_sec.cmp(&other.tv_sec) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.tv_usec.cmp(&other.tv_usec),
            Ordering::Greater => Ordering::Greater,
        }
    }
}

impl PartialOrd for TimeVal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct TimeZone {
    pub tz_minuteswest: u32,
    pub tz_dsttime: u32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ITimerVal {
    pub it_interval: TimeVal,
    pub it_value: TimeVal,
}
impl ITimerVal {
    pub fn new() -> Self {
        Self {
            it_interval: TimeVal::new(),
            it_value: TimeVal::new(),
        }
    }
}

#[derive(Clone, Copy)]
/// Store the current process times used in the `time()`.
#[repr(C)]
pub struct Times {
    /// user time
    pub tms_utime: usize,
    /// system time
    pub tms_stime: usize,
    /// user time of children
    pub tms_cutime: usize,
    /// system time of children
    pub tms_cstime: usize,
}

pub enum TimeRange {
    TimeSpec(TimeSpec),
    TimeVal(TimeVal),
}
