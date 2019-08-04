# Timer RusPiRo crate

This crate provides simple functions to pause execution on the current core for a given amount of time. It uses the
free-running counter of the Raspberry Pi to provide micro second accurate pause timings.

[![Travis-CI Status](https://api.travis-ci.org/RusPiRo/ruspiro-timer.svg?branch=master)](https://travis-ci.org/RusPiRo/ruspiro-timer)
[![Latest Version](https://img.shields.io/crates/v/ruspiro-timer.svg)](https://crates.io/crates/ruspiro-timer)
[![Documentation](https://docs.rs/ruspiro-timer/badge.svg)](https://docs.rs/ruspiro-timer)
[![License](https://img.shields.io/crates/l/ruspiro-timer.svg)](https://github.com/RusPiRo/ruspiro-timer#license)

## Usage
To use the crate just add the following dependency to your ``Cargo.toml`` file:
```
[dependencies]
ruspiro-timer = "0.1.0"
```

Once done the access to the timer functions is available in your rust files like so:
```
use rusprio_timer as timer;

fn demo() {
    timer::sleep(1000); // pause for 1 milli second
    timer::sleepcycles(200); // pause for 200 CPU cycles
}
```

## License
Licensed under Apache License, Version 2.0, ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)