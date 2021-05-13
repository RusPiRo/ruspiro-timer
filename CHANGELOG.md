# Changelog

## :peach: v0.5.1

- ### :bulb: Features

  - Re-export `Duration` type from `core::time` for convinience when using this crate.
  
## :peach: v0.5.0

- ### :wrench: Maintenance

  - Migrate the build pipeline to github actions
  - update versions of dependent crates

- ### :bulb: Features

  - Use the `Duration` type from `core::time` module instead of custom duration type.

## :banana: v0.4.1

- ### :detective: Fixes

  - remove `asm!` macro usages and replace with `llvm_asm!`
  - use `cargo make` to stabilize cross-platform builds

## :pizza: v0.4.0

- ### :bulb: Features

  - Introduce the possibility to schedule function/closure execution with a delay in multiples of milli seconds relative to the current time of execution.

- ### :wrench: Maintenance

  - Some code refactoring for the new functionality and hopefully cleaner structure
