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
/*
extern crate alloc;
use alloc::{
    boxed::Box,
    vec::Vec,
    collections::binary_heap::BinaryHeap,
};
use core::sync::atomic::*;
use ruspiro_interrupt::*;
use ruspiro_singleton::Singleton;

use ruspiro_console::*;
*/
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


/*
// the singleton wrapped heapmap of the functions that need to be executed
// a specific implementation of Ord ensured that we are running a min-heap and not the default
// max-heap
static TIMERHANDLER: Singleton<Option<Vec<ScheduledFn>>> = Singleton::new(Some(Vec::new()));
// storing the offset into the heapmap to skip already called handler
static OFFSET: AtomicUsize = AtomicUsize::new(0);

/// Schedule a function/closure to be executed with the given delay in micro seconds
/// # Example
/// ```no_run
/// # use ruspiro_timer::*;
/// # fn doc() {
/// schedule(1000, || println!("scheduled execution"));
/// # }
/// ```
/// 
/// ```no_run
/// # use ruspiro_timer::*;
/// # fn doc() {
/// let mut v = 100;
/// schedule(1000, move || println!("scheduled execution, v was: {}", v));
/// v += 10;
/// println!("v is: {}", v);
/// # }
/// ```
pub fn schedule<F: Fn() + 'static + Send>(delay: u32, function: F) {
    // from the current free running counter and the requested delay get
    // the due value that need's to be put into the compare register to trigger
    // the timer interrupt for this scheduled function
    let due = now() + delay as u64;
    TIMERHANDLER.take_for(|timed_handler: &mut Option<Vec<ScheduledFn>>| {
        // this is the only place where we could mutably access the TIMERHANDLER Vec
        // create a BinaryHeap from this vec to add sorted new scheduled functions
        let handler = timed_handler.take();
        if let Some(handler) = handler {
            let mut heap = BinaryHeap::from(handler);
            heap.push(ScheduledFn {
                due,
                func: Box::new(function)
            });
            
            let handler = heap.into_vec();

            // once we have pushed a new schedule peek the one that need to be scheduled next
            // to ensure the timer interrupt match value is updated if a new function is scheduled
            // in less time as the currently known next one
            let offset = OFFSET.load(Ordering::Acquire);
            let next_due = handler[offset].due;
            println!("set next {} match value to {}", offset, next_due as u32);
            // when casting due to u32 the u32 rollover that was happening and we only cut the upper 32 bits
            // is much appreciated as this is how the timer works...
            SYS_TIMERC1::Register.set(next_due as u32);
        
            timed_handler.replace(handler);
        };

        println!("new function scheduled");
    });
    // clear the match flag from the control register before setting a new match value
    // otherwise the interrupt might be immediately triggered when activated even if no match
    // happened yet...
    SYS_TIMERCS::Register.write_value(SYS_TIMERCS::M1::MATCH);
    // if not already done we need to activate the system timer interrupt
    IRQ_MANAGER.take_for(|mgr: &mut InterruptManager| mgr.activate(Interrupt::SystemTimer1));
    println!("irq activated");
}

/// Implement the timer interrupt handler for interrupt based timed execution
#[IrqHandler(SystemTimer1)]
fn timer_handler() {
    println!("times-up");
    if SYS_TIMERCS::Register.read(SYS_TIMERCS::M1) == 1 {
        // first acknowledge the timer interrupt by writing 1 to the match register value
        SYS_TIMERCS::Register.write_value(SYS_TIMERCS::M1::MATCH);
        
        // now execute the scheduled function, ensuring we do not take any exclusive locks
        TIMERHANDLER.use_for(|timed_handler| {
            timed_handler.as_ref().map(|handler| {
                // get the current offset into the handler list beeing the next to be executed
                // and increasing the same
                let offset = OFFSET.fetch_add(1, Ordering::AcqRel);
                // now call the scheduled function at this offset
                println!("call scheduled function {} due {} at {}", offset, handler[offset].due, now());
                (handler[offset].func)();
                println!("scheduled function finished");
                // after calling this check for the next available to set the new schedule time
                // for the interrupt
                if offset+1 < handler.len() {
                    println!("set timer for next schedule");
                    SYS_TIMERC1::Register.set(handler[offset+1].due as u32);
                };
            });
        });
    }
    println!("timer irq done");
}
*/
