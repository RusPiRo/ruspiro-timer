# Timer RusPiRo crate

This crate provides simple functions to pause execution on the current core for a given amount of time. It uses the
free-running counter of the Raspberry Pi to provide micro second accurate pause timings.

![CI](https://github.com/RusPiRo/ruspiro-timer/workflows/CI/badge.svg?branch=development)
[![Latest Version](https://img.shields.io/crates/v/ruspiro-timer.svg)](https://crates.io/crates/ruspiro-timer)
[![Documentation](https://docs.rs/ruspiro-timer/badge.svg)](https://docs.rs/ruspiro-timer)
[![License](https://img.shields.io/crates/l/ruspiro-timer.svg)](https://github.com/RusPiRo/ruspiro-timer#license)

## Features

Feature         | Description
----------------|------------------------------------------------------------------------------
``ruspiro_pi3`` | active to use the proper timer MMIO base memory address for Raspberry Pi 3 when accessing the system timer peripheral

## Usage

To use the crate just add the following dependency to your ``Cargo.toml`` file:

```toml
[dependencies]
ruspiro-timer = "||VERSION||"
```

Once done the access to the timer functions is available in your rust files like so:

```rust
use core::time::Duration
use rusprio_timer:*;

fn foo() {
    sleep(Duration::from_millis(1)); // pause for 1 millisecond
    sleepcycles(200); // pause for 200 CPU cycles
}
```

Scheduling the execution of a function/closure is as simple as this:

```rust
use core::time::Duration;
use ruspiro_timer::*;

fn foo() {
    // print after 100 milliseconds
    schedule(Duration:from_millis(100), || println!("delayed execution"));
}
```

## License

Licensed under Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0) or MIT ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)) at your choice.
