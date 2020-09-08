/***************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-timer/0.4.0")]
#![cfg_attr(not(any(test, doctest)), no_std)]
#![feature(llvm_asm)]
//! # Timer functions
//!
//! This crate provides simple timing functions to pause the actual core for a specific amount of time.
//! It is also possible to delay function/closure execution. This is based on system timer interrupts.
//!
//!
//! # Features
//! Feature         | Description
//! ----------------|------------------------------------------------------------------------------
//! ``ruspiro_pi3`` | active to use the proper timer MMIO base memory address for Raspberry Pi 3 when accessing the system timer peripheral
//!

use ruspiro_register::system::nop;

mod interface;
use interface::*;

mod schedule;
pub use schedule::schedule;

/// Type representing micro-seconds
#[derive(Copy, Clone)]
pub struct Useconds(pub u64);

/// Type representing milli-seconds
#[derive(Copy, Clone)]
pub struct Mseconds(pub u64);

/// Pause the current execution for the given amount of micro seconds
/// # Example
/// ```no_run
/// # use ruspiro_timer::*;
/// # fn doc() {
/// // pause the execution for 1 second
/// sleep(Useconds(1_000_000));
/// # }
/// ```
pub fn sleep(usec: Useconds) {
    let wait_until = Useconds(now().0 + usec.0);

    while !is_due(wait_until) {}
}

/// Pause the current execution for the given amount of CPU cycles
/// # Example
/// ```no_run
/// # use ruspiro_timer::*;
/// # fn doc() {
/// sleepcycles(1_000);
/// # }
/// ```
pub fn sleepcycles(cycles: u32) {
    for _ in 0..cycles {
        nop();
    }
}

/// Get the current time as free running counter value of the system timer
pub fn now() -> Useconds {
    let t_low = SYS_TIMERCLO::Register.get() as u64;
    let t_high = SYS_TIMERCHI::Register.get() as u64;

    Useconds((t_high << 32) | t_low)
}

/// Compare the given time as free running counter value with the current time.
/// Returns true if the current time is later than the time passed into this function.
fn is_due(time: Useconds) -> bool {
    if time.0 == 0 {
        // if no valid time is given, time is always due
        true
    } else {
        // returns true if we have reached the current time (counter)
        now().0 >= time.0
    }
}
