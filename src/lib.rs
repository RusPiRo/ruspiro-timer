/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/
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
extern crate alloc;
use alloc::{
    vec::Vec,
    boxed::Box,
    collections::binary_heap::BinaryHeap,
};
use ruspiro_register::{define_mmio_register, system::nop};
use ruspiro_interrupt::*;
use ruspiro_singleton::Singleton;

use ruspiro_console::*;

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
fn now() -> Useconds {
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

// the structure storing functions that should be executed at a specific time
struct ScheduledFn {
    due: u64,
    pub func: Box<dyn FnOnce() + 'static + Send>,
}

impl core::cmp::Eq for ScheduledFn {}
impl core::cmp::PartialEq for ScheduledFn {
    fn eq(&self, other: &ScheduledFn) -> bool {
        self.due == other.due
    }
}

// implement Ord for the scheduled functions and reverse the compare order to get a min-heap
impl core::cmp::Ord for ScheduledFn {
    fn cmp(&self, other: &ScheduledFn) -> core::cmp::Ordering {
        other.due.cmp(&self.due)
    }
}

impl core::cmp::PartialOrd for ScheduledFn {
    fn partial_cmp(&self, other: &ScheduledFn) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// the singleton wrapped heapmap of the functions that need to be executed
// a specific implementation of Ord ensured that we are running a min-heap and not the default
// max-heap
static TIMERHANDLER: Singleton<Option<BinaryHeap<ScheduledFn>>> = Singleton::new(None);

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
pub fn schedule<F: FnOnce() + 'static + Send>(delay: u32, function: F) {
    // from the current free running counter and the requested delay get
    // the due value that need's to be put into the compare register to trigger
    // the timer interrupt for this scheduled function
    let due = now() + delay as u64;
    TIMERHANDLER.take_for(|maybe_handler: &mut Option<BinaryHeap<ScheduledFn>>| {
        // as BinaryHeap::new() is not a const fn we need to put it behind an [Option] to be able to
        // use it inside the ``static`` definition of the handler list. But this requires this 
        // "fancy" trick to create an empty BinaryHeap if it has not been there at the first call
        let mut handler = if let Some(handler) = maybe_handler.take() {
            handler
        } else {
            println!("create new heap");
            BinaryHeap::new()
        };

        handler.push(
            ScheduledFn {
                due,
                func: Box::new(function),
            }
        );

        // once we have pushed a new schedule peek the one that need to be scheduled next
        // to ensure the timer interrupt match value is updated if a new function is scheduled
        // in less time as the currently known next one
        if let Some(next_due) = handler.peek().map(|next| next.due) {
            println!("set next match value to {}", next_due as u32);
            // when casting due to u32 the u32 rollover that was happening and we only cut the upper 32 bits
            // is much appreciated as this is how the timer works...
            SYS_TIMERC1::Register.set(next_due as u32);    
        }

        // now that we know we have a [BinaryHeap] instance put it back into the [Option]
        maybe_handler.replace(handler);
    });
    // clear the match flag from the control register before setting a new match value
    // otherwise the interrupt might be immediately triggered when activated even if no match
    // happened yet...
    SYS_TIMERCS::Register.write_value(SYS_TIMERCS::M1::MATCH);
    // if not already done we need to activate the system timer interrupt
    IRQ_MANAGER.take_for(|mgr: &mut InterruptManager| mgr.activate(Interrupt::SystemTimer1));
}

/// Implement the timer interrupt handler for interrupt based timed execution
#[IrqHandler(SystemTimer1)]
fn timer_handler() {
    if SYS_TIMERCS::Register.read(SYS_TIMERCS::M1) == 1 {
        // first acknowledge the timer interrupt by writing 1 to the match register value
        SYS_TIMERCS::Register.write_value(SYS_TIMERCS::M1::MATCH);
        // now execute the scheduled function
        TIMERHANDLER.take_for(
            |maybe_list| {
                if let Some(ref mut list) = maybe_list {
                    if let Some(next_schedule) = list.pop() {
                        println!("call scheduled function due {} at {}", next_schedule.due, now());
                        (next_schedule.func)();
                        // once this has been executed peek the next scheduled item from the heap to calculate
                        // the next time the interrupt need to be raised
                        if let Some(next_due) = list.peek().map(|next| next.due) {
                            SYS_TIMERC1::Register.set(next_due as u32);
                        }
                    };/* else {
                        // as there is no more function registered de-activate the timer interrupt
                        IRQ_MANAGER.take_for(|mgr: &mut InterruptManager| mgr.deactivate(Interrupt::SystemTimer1));
                    }*/
                }
            }
        );
    }
}

// MMIO peripheral base address based on the target family provided with the custom target config file.
#[cfg(feature = "ruspiro_pi3")]
const PERIPHERAL_BASE: u32 = 0x3F00_0000;

// Base address of system timer MMIO register
const SYS_TIMER_BASE: u32 = PERIPHERAL_BASE + 0x3000;
// Base address of ARM timer MMIO register
const ARM_TIMER_BASE: u32 = PERIPHERAL_BASE + 0xB000;

// Define the MMIO timer register
define_mmio_register![
    /// system timer control register, keep in mind that actually only timer 1 and 3 are free on RPi
    SYS_TIMERCS<ReadWrite<u32>@(SYS_TIMER_BASE)> {
        /// system timer 0 match flag
        M0 OFFSET(0) [
            MATCH = 1,
            CLEAR = 0
        ],
        /// system timer 1 match flag
        M1 OFFSET(1) [
            MATCH = 1,
            CLEAR = 0
        ],
        /// system timer 2 match flag
        M2 OFFSET(2) [
            MATCH = 1,
            CLEAR = 0
        ],
        /// system timer 3 match flag
        M3 OFFSET(3) [
            MATCH = 1,
            CLEAR = 0
        ]
    },
    /// system timer free running counter lower 32Bit value
    SYS_TIMERCLO<ReadOnly<u32>@(SYS_TIMER_BASE + 0x04)>,
    /// system timer free running counter higher 32Bit value
    SYS_TIMERCHI<ReadOnly<u32>@(SYS_TIMER_BASE + 0x08)>,
    /// system timer compare value register
    SYS_TIMERC0<ReadWrite<u32>@(SYS_TIMER_BASE + 0x0C)>,
    SYS_TIMERC1<ReadWrite<u32>@(SYS_TIMER_BASE + 0x10)>,
    SYS_TIMERC2<ReadWrite<u32>@(SYS_TIMER_BASE + 0x14)>,
    SYS_TIMERC3<ReadWrite<u32>@(SYS_TIMER_BASE + 0x18)>,

    /// ARM timer load value that is put into the value register once it counted to 0
    pub ARM_TIMERLOAD<ReadWrite<u32>@(ARM_TIMER_BASE + 0x400)>,
    /// ARM timer current counter value
    pub ARM_TIMERVALUE<ReadOnly<u32>@(ARM_TIMER_BASE + 0x404)>,
    /// ARM timer control register
    pub ARM_TIMERCTRL<ReadWrite<u32>@(ARM_TIMER_BASE + 0x408)> {
        /// width of the timer counter values
        WIDTH OFFSET(1) [
            _16Bit = 0,
            _32Bit = 1
        ],
        /// pre-scaler bits
        PRESCALER OFFSET(2) BITS(2) [
            CLOCK_DIV_1 = 0b00,
            CLOCK_DIV_16 = 0b01,
            CLOCK_DIV_256 = 0b10
        ],
        /// flag to enable timer interrupts beein raised
        IRQ OFFSET(5) [
            ENABLED = 1,
            DISABLED = 0
        ],
        /// flag to enable the ARM timer
        TIMER OFFSET(7) [
            ENABLED = 1,
            DISABLED = 0
        ],
        /// flag to indicate if timer should stop or keep running in debug halted mode
        DEBUG OFFSET(8) [
            STOP = 1,
            RUN = 0
        ],
        /// flag to enable the free-running counter
        FREERUN OFFSET(9) [
            ENABLED = 1,
            DISABLED = 0
        ],
        /// free running counter pre-scaler = FREQUENCY = SYS_CLOCK/(FR_PRESCALER+1)
        FR_PRESCALER OFFSET(16) BITS(8)
    },
    /// ARM timer interrupt acknowledge register
    pub ARM_TIMERACKN<WriteOnly<u32>@(ARM_TIMER_BASE + 0x40C)>,
    pub ARM_TIMERRAWIRQ<ReadOnly<u32>@(ARM_TIMER_BASE + 0x410)> {
        PENDING OFFSET(0) [
            SET = 1,
            CLEAR = 0
        ]
    },
    /// masked interrupt assertion value (defacto = RAWIRQ logical AND IRQENABLE)
    pub ARM_TIMERMASKIRQ<ReadOnly<u32>@(ARM_TIMER_BASE + 0x414)> {
        FLAG OFFSET(0) [
            NOT_ASSERTED = 0,
            ASSERTED = 1
        ]
    },
    /// ARM timer pre-devide value, timer_clock = apb_clock/(pre_devider + 1), default value = 0x7d (125), gives a divide by 126
    pub ARM_TIMERPREDEV<ReadWrite<u32>@(ARM_TIMER_BASE + 0x41C)> {
        VALUE OFFSET(0) BITS(10)
    },
    /// ARM timer free running counter value
    pub ARM_TIMERFRCOUNTER<ReadOnly<u32>@(ARM_TIMER_BASE + 0x420)>
];