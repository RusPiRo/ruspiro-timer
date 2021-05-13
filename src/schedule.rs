/***************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **************************************************************************************************/

//! # Schedule Functions
//!
//! Allowing functions/closures to be scheduled with a specific delay. The scheduling is based on
//! timer interrupts. When using this functionality ensure the interrupts are properly initialized
//! and globally activated using the [``ruspiro_interrupt`` crate](https://crates.io/crates/ruspiro_interrupt)
//!

extern crate alloc;
use crate::*;
use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};
use core::{
  cell::UnsafeCell,
  sync::atomic::{AtomicU64, AtomicUsize, Ordering},
  time::Duration,
};
use ruspiro_interrupt::{self as irq, Interrupt, IrqHandler, IsrSender};
use ruspiro_singleton::*;

type FunctionScheduleList =
  BTreeMap<Duration, UnsafeCell<Option<Box<dyn FnOnce() + 'static + Send>>>>;

/// Structure to contain the data needed to "manage" the functions to be scheduled
struct Schedules {
  /// Timer value for the very next function to be executed
  pub next_due: AtomicU64,
  /// Sorted list of function to be executed. The key is the timer value when they are due
  pub schedule_list: FunctionScheduleList,
  /// index into the schedule list pointing to the next due entry that will be executed once
  /// the timer interrupt is triggered the next time
  pub due_index: AtomicUsize,
  /// index into the schedule list pointing to the last already done entry. This is used to check
  /// whether it would be safe to shrink the schedule list to get rid of all the already executed
  /// functions to reduce memory consumption
  pub done_index: AtomicUsize,
}

impl Schedules {
  /// Create a new instance holding the schedule information
  fn new() -> Self {
    Self {
      next_due: AtomicU64::new(0),
      schedule_list: BTreeMap::new(),
      due_index: AtomicUsize::new(0),
      done_index: AtomicUsize::new(0),
    }
  }

  /// Shrink the list of scheduled functions to get rid of all what has been processed already.
  ///
  /// # Safety
  /// This is safe when this function is called when it is ensured that no concurrent processing
  /// tries actually to index into the values or keys of the list, while shrinking. A typical
  /// scenario would be, when we are about to add a new entry to the list and see that the index
  /// for done items is equal to the index of due items, which means that there will be no interrupt
  /// triggered that may want to execute a scheduled function.
  unsafe fn shrink(&mut self) {
    self.schedule_list.clear();
    // after removing the scheduled functions we can reset the due and done indices
    // as well as the next due value to ensure the first scheduled function will set the
    // appropriate value
    self.next_due.store(0, Ordering::Release);
    self.due_index.store(0, Ordering::Release);
    self.done_index.store(0, Ordering::Release);
  }
}

// TODO: is it really safe to say Schedules are Send and Sync just the way they are defined?
unsafe impl Send for Schedules {}
unsafe impl Sync for Schedules {}

/// The global static carrying the list of scheduled functions. The type looks a bit arkward at first
/// look but is needed to fulfill the following reqirements and constrains
/// 1. We need mutual exclusive access to the sorted list to add new scheduled functions to it
/// 2. Mutual exclusive access should not be needed while inside the interrupt handler to circumvent
///    deadlock situations
/// 3. Even though the interrupt handler has no mutual exclusive access to the whole list it would need
///    mutual exclusive access to the stored function to consume it while calling
/// 4. As the BTreeMap's new function is not a const one we need to wrap it with a Option
static SCHEDULE: Singleton<Option<Schedules>> = Singleton::new(None);

/// Schedule a function for delayed execution with a millisecond offset relative to the time of the
/// execution of this function.
/// ## Hint:
/// The function scheduled will be executed in the context of the system
/// timer interrupt, so heavy computation should be avoided. However, it could be used to signal that
/// a heavy processing can continue in the context outside the actual interrupt using a ``Semaphore``
/// or a ``Channel``
///
/// # Example
/// ```no_run
/// # use ruspiro_timer::*;
/// # fn doc() {
///     // use a simple counter variable to be passed to the scheduled function
///     let mut counter = 10;
///     // schedule a function that prints the value of the variable at the time of beeing scheduled
///     // after 1 second
///     schedule(Mseconds(1_000), move || println!("Value when scheduled: {}", counter));
///     counter += 10;
///     // print the actual value of the variable as processing continues
///     println!("actual value: {}", counter);
///     // sleep to wait for the scheduled function to get executed (this is in micro seconds!)
///     sleep(Useconds(1_500_000));
/// # }
/// ```
/// The expected output of this example would be:
/// ```ignore
/// actual value: 20
/// Value when scheduled: 10
/// ```
pub fn schedule<F: FnOnce() + 'static + Send>(delay: Duration, function: F) {
  // calculate the time this function shall be scheduled based on the current time and the
  // requested delay given in milli seconds
  let due = now() + delay;
  // take the list and add the new entry
  SCHEDULE.with_mut(|schedules: &mut Option<Schedules>| {
    if schedules.is_none() {
      // when the first function get's to be scheduled create the new sorted list
      schedules.replace(Schedules::new());
      // than clear the match flag from the control register after otherwise the interrupt might
      // be immediately triggered when activated as the initial value might immidiately match the
      // timer value ...
      SYS_TIMERCS::Register.write_value(SYS_TIMERCS::M1::MATCH);
      // and activate the timer interrupts to be dispatched
      irq::activate(Interrupt::SystemTimer1, None);
    }

    if let Some(ref mut schedules) = schedules.as_mut() {
      // before inserting a new scheduled function check if we could shrink the list
      // get the last due and done index
      let due_index = schedules.due_index.load(Ordering::Relaxed) - 1;
      let done_index = schedules.done_index.load(Ordering::Relaxed);
      // if something has been done already and we are done with all that have been due it is
      // safe to shrink the list
      if done_index > 0 && due_index == done_index {
        // as we have mutual exclusive access here there is no other way items could be added
        // so once the done index equals the due index we can safely shrink the list
        unsafe {
          schedules.shrink();
        }
      };

      schedules
        .schedule_list
        .insert(due, UnsafeCell::new(Some(Box::new(function))));
      // now that we have added the new function check if we need to adjust the already set match
      // value for the interrupt to be raised
      let next_due = Duration::from_micros(schedules.next_due.load(Ordering::Acquire));
      // on first entry, when the current next due is after the new due
      // or when the current next_due is already in the past, set a new next due
      if next_due.is_zero() || due < next_due || next_due < now() {
        schedules
          .next_due
          .store(due.as_micros() as u64, Ordering::Release);
        SYS_TIMERC1::Register.set(due.as_micros() as u32);
      };
    };
  });
}

/// Implement the timer interrupt handler for interrupt based timed execution
#[IrqHandler(SystemTimer1)]
unsafe fn timer_handler(tx: Option<IsrSender<Box<dyn Any>>>) {
  // check which timer compare/match value has raised this interrupt, only care on number 1 ...
  if SYS_TIMERCS::Register.read(SYS_TIMERCS::M1) == 1 {
    // first acknowledge the timer interrupt by writing 1 to the match register value
    SYS_TIMERCS::Register.write_value(SYS_TIMERCS::M1::MATCH);
    // use the list to find the the entry we should execute now, as it is sorted we start from
    // the front, the actual index into the list is atomically stored to ensure even we can not
    // have mutual exclusive access to the list
    SCHEDULE.with_ref(|schedules: &Option<Schedules>| {
      if let Some(ref schedules) = schedules {
        let next_idx = schedules.due_index.fetch_add(1, Ordering::AcqRel);
        if next_idx >= schedules.schedule_list.len() {
          return;
        }

        let functions: Vec<_> = schedules.schedule_list.values().collect();
        let function_cell = functions[next_idx];
        // now we have the cell containing the function to be called
        // accessing this mutably is safe as we are now the only one accessing this entry
        // due to the fact that we have atomically adjusted the index into the list, so any
        // other core will use a different index...
        let function = function_cell.get();
        // take the function out of the option
        let function_to_call = (*function).take().unwrap();
        // call the function
        (function_to_call)();
        // in case there is already another function scheduled in the list retrieve it's due
        // time and setup the next match value
        if schedules.schedule_list.len() > next_idx + 1 {
          // this is safe here as the list of scheduled functions only grows with one
          // exception, when the a new entry is about to e added while all other are already
          // processed
          let due_list: Vec<_> = schedules.schedule_list.keys().collect();
          let next_due = due_list[next_idx + 1];
          // TOCHECK: setting the next due from the list contains a small uncertainty as this
          // interrupt might have interferred the insertion of a scheduled functions that was
          // scheduled with a due time smaller than the one just retrieved from the list
          // this lead to a very tiny possibility that the next trigger value is not set
          // properly. However, as scheduling is only possible with a minimal delay of 1ms
          // this window, smaller than a micro-second should never occur
          SYS_TIMERC1::Register.set(next_due.as_micros() as u32);
          schedules
            .next_due
            .store(next_due.as_micros() as u64, Ordering::SeqCst);
        }
        // as we have executed this function and are don with all related updates we can update
        // the index of the done functions
        schedules.done_index.store(next_idx, Ordering::Release);
      }
    });
  }
}
