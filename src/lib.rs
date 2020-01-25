/***************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-timer/0.4.0")]
#![cfg_attr(not(any(test, doctest)), no_std)]
#![feature(asm)]
//! # Timer functions
//!
//! This crate provides simple timing functions to pause the actual core for a specific amount of time.
//!
//! # Usage
//!
//! ```no_run
//! use ruspiro_timer as timer;
//!
//! fn foo() {
//!     timer::sleep(1000); // pause for 1 milli second
//!     timer::sleepcycles(200); // pause for 200 CPU cycles
//! }
//!
//! ```
//!
//! # Features
//!
//! - ``ruspiro_pi3`` is active by default and ensures the proper timer MMIO base memory address is
//! used for Raspberry Pi 3
//!

mod interface;
use interface::*;

mod schedule;
pub use schedule::schedule;

use ruspiro_register::system::nop;

pub type Useconds = u64;

/// Pause the current execution for the given amount of micro seconds
pub fn sleep(usec: Useconds) {
    let wait_until = now() + usec;

    while !is_due(wait_until) {}
}

/// Pause the current execution for the given amount of CPU cycles
pub fn sleepcycles(cycles: u32) {
    for _ in 0..cycles {
        nop();
    }
}

/// Get the current time as free running counter value of the system timer
pub fn now() -> Useconds {
    let t_low = SYS_TIMERCLO::Register.get() as u64;
    let t_high = SYS_TIMERCHI::Register.get() as u64;

    (t_high << 32) | t_low
}

/// Compare the given time as free running counter value with the current time.
/// Returns true if the current time is later than the time passed into this function.
fn is_due(time: Useconds) -> bool {
    if time == 0 {
        // if no valid time is given, time is always due
        true
    } else {
        // returns true if we have reached the current time (counter)
        now() >= time
    }
}
