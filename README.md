# Timer RusPiRo crate

This crate provides simple functions to pause execution on the current core for a given amount of time. It uses the
free-running counter of the Raspberry Pi to provide micro second accurate pause timings.

[![Travis-CI Status](https://api.travis-ci.org/RusPiRo/ruspiro-timer.svg?branch=master)](https://travis-ci.org/RusPiRo/ruspiro-timer)
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
ruspiro-timer = "0.4"
```

Once done the access to the timer functions is available in your rust files like so:
```rust
use rusprio_timer:*;

fn foo() {
    sleep(Useconds(1_000)); // pause for 1 millisecond
    sleepcycles(Useconds(200)); // pause for 200 CPU cycles
}
```

Scheduling the execution of a function/closure is as simple as this:
```rust
use ruspiro_timer::*;

fn foo() {
    schedule(Mseconds(100), || println!("delayed execution")); // print after 100 milliseconds
}
```

## License
Licensed under Apache License, Version 2.0, ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)