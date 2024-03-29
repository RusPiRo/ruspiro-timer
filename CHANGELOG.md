# Changelog

## :cat: v0.6.0

Introduce the MMIO address mapping to support also Raspberry 4 as a target for this crate.

- ### :bulb: Features

  - add features `pi4_low` and `pi4_high` to compile for Raspberry Pi 4 model

- ### :wrench: Maintenance

  - rename `ruspiro_pi3` feature to `pi3`

## :melon: v0.5.2

This is a maintenance release ensuring successful build with the latest nightly (2021-09-05) version of Rust.

- ### :wrench: Maintenance

  - build the current crate with the latest nightly Rust version
  - bump the dependency versions
  - remove the unsused `llvm_asm` feature

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
