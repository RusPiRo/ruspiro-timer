/***************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-timer/||VERSION||")]
#![cfg_attr(not(any(test, doctest)), no_std)]
//! # Timer functions
//!
//! This crate provides simple timing functions to pause the actual processing for a specific amount of time. The core
//! pausing is called on will block.
//!
//! It is also possible to delay function/closure execution. This is based on system timer interrupts.
//!
//!
//! # Features
//! Feature       | Description
//! --------------|------------------------------------------------------------------------------
//! `ruspiro_pi3` | active to use the proper timer MMIO base memory address for Raspberry Pi 3 when accessing the system timer peripheral
//!

#[cfg(not(any(feature = "pi3", feature = "pi4_lowperi", feature = "pi4_highperi")))]
compile_error!("Either feature \"pi3\", \"pi4_lowperi\" or \"pi4_highperi\" must be enabled for this crate");

extern crate alloc;

mod interface;
mod schedule;
pub use schedule::schedule;

pub use core::time::Duration; // re-export Duration for convinence when using this crate
use interface::*;
use ruspiro_arch_aarch64::instructions::nop;

/// Pause the current execution for the given amount of micro seconds
/// # Example
/// ```no_run
/// # use ruspiro_timer::*;
/// # fn doc() {
/// // pause the execution for 1 second
/// sleep(Duration::from_secs(1));
/// # }
/// ```
pub fn sleep(duration: Duration) {
  let wait_until = now() + duration;

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
/// # Example
/// ```no_run
/// # use ruspiro_timer::*;
/// # fn doc() {
/// let now = now();
/// # }
/// ```
pub fn now() -> Duration {
  let t_low = SYS_TIMERCLO::Register.get() as u64;
  let t_high = SYS_TIMERCHI::Register.get() as u64;

  Duration::from_micros((t_high << 32) | t_low)
}

/// Compare the given time as free running counter value with the current time.
/// Returns true if the current time is later than the time passed into this function.
/// # Example
/// ```no_run
/// # use ruspiro_timer::*;
/// # fn doc() {
/// let due_time = now() + Duration::from_secs(100);
/// if is_due(due_time) {
///     println!("Time is due :)");    
/// }
/// # }
/// ```
fn is_due(time: Duration) -> bool {
  if time == Duration::from_micros(0) {
    // if no valid time is given, time is always due
    true
  } else {
    // returns true if we have reached the current time (counter)
    now() >= time
  }
}
