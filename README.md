# Timer RusPiRo crate

This crate provides simple functions to pause execution on the current core for a given amount of time. It uses the
free-running counter of the Raspberry Pi to provide micro second accurate pause timings.

## Usage
To use the crate just add the following dependency to your ``Cargo.toml`` file:
```
[dependencies]
ruspiro-timer = "0.0.1"
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