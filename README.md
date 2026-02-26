# FT6336U Capacitive Touch Controller Driver (ft6336u-dd)

[![Crates.io](https://img.shields.io/crates/v/ft6336u-dd.svg)](https://crates.io/crates/ft6336u-dd)
[![Docs.rs](https://docs.rs/ft6336u-dd/badge.svg)](https://docs.rs/ft6336u-dd)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses)
[![Build Status](https://img.shields.io/github/actions/workflow/status/okhsunrog/ft6336u-dd/rust_ci.yml?logo=github)](https://github.com/okhsunrog/ft6336u-dd/actions/workflows/rust_ci.yml)

This crate provides a `no_std` driver for the FocalTech FT6336U capacitive touch controller, a self-capacitance touch panel controller supporting up to 2 simultaneous touch points. The driver leverages the [`device-driver`](https://crates.io/crates/device-driver) crate with a declarative YAML manifest ([`device.yaml`](device.yaml)) for a type-safe register map definition covering 27 registers.

## Overview

The `ft6336u-dd` driver offers:

- **Declarative Configuration:** The FT6336U register map is defined in [`device.yaml`](device.yaml), enabling `device-driver` to generate a type-safe, low-level register access API.
- **Unified Async/Blocking API:** Uses the [`bisync`](https://github.com/JM4ier/bisync) crate to provide both asynchronous (`Ft6336uAsync`) and blocking (`Ft6336u`) drivers from the same codebase, with no feature flags required.
- **High-Level and Low-Level APIs:**
  - High-level methods simplify tasks like scanning touch points and configuring thresholds.
  - Low-level API (via the `ll` field) offers direct, type-safe access to all registers defined in `device.yaml`.
- **Efficient I2C:** `scan()` reads 2 touch points in a single 13-byte I2C transaction.
- **`no_std` and `no-alloc`:** Optimized for bare-metal and RTOS environments.
- **Optional Logging:** Supports `defmt` and the `log` facade for debugging.

## Getting Started

1. **Add `ft6336u-dd` to `Cargo.toml`:**

   ```toml
   [dependencies]
   ft6336u-dd = "0.1.0"
   # For blocking usage (Ft6336u):
   embedded-hal = "1.0.0"
   # For async usage (Ft6336uAsync):
   embedded-hal-async = "1.0.0"
   ```

   > **Note:** Add the relevant `embedded-hal` crate for your use case, no need for both.

2. **Instantiate the driver and scan for touches:**

   - **Blocking:**
     ```rust
     use ft6336u_dd::{Ft6336u, TouchStatus};

     let mut touch = Ft6336u::new(i2c);

     let data = touch.scan()?;
     for point in &data.points {
         if point.status != TouchStatus::Release {
             // Handle touch at (point.x, point.y)
         }
     }
     ```

   - **Async:**
     ```rust
     use ft6336u_dd::{Ft6336uAsync, TouchStatus};

     let mut touch = Ft6336uAsync::new(i2c);

     let data = touch.scan().await?;
     for point in &data.points {
         if point.status != TouchStatus::Release {
             // Handle touch at (point.x, point.y)
         }
     }
     ```

### Configuration

```rust
use ft6336u_dd::PowerModeEnum;

// Set touch sensitivity (lower = more sensitive)
touch.write_touch_threshold(40)?;

// Set report rate in active mode (Hz)
touch.write_active_rate(60)?;

// Enter hibernate mode
touch.write_power_mode(PowerModeEnum::Hibernate)?;
```

## Low-Level API Usage

The driver provides direct access to all FT6336U registers through the low-level API via `touch.ll`. This API is automatically generated from [`device.yaml`](device.yaml) and provides type-safe access to all register fields.

### Reading Registers

Use `.read()` to read a register and access its fields:

```rust
// Read chip ID
let chip_id = touch.ll.chip_id().read()?;
assert_eq!(chip_id.value(), 0x64); // FT6336U

// Read power mode
let power = touch.ll.power_mode().read()?;
let mode = power.mode(); // Returns PowerModeEnum
```

### Writing Registers

Use `.write()` with a closure to set register fields:

```rust
// Set touch threshold
touch.ll.threshold().write(|w| {
    w.set_value(40);
})?;

// Set interrupt mode to trigger
touch.ll.g_mode().write(|w| {
    w.set_mode(GestureMode::Trigger);
})?;
```

### Modifying Registers

Use `.modify()` to read-modify-write, preserving other fields:

```rust
touch.ll.device_mode().modify(|w| {
    w.set_mode(DeviceMode::Working);
})?;
```

### Async Low-Level API

The low-level API has async versions for use with `Ft6336uAsync`. Simply append `_async` to the method name:

```rust
let chip_id = touch.ll.chip_id().read_async().await?;

touch.ll.threshold().write_async(|w| {
    w.set_value(40);
}).await?;
```

### Finding Register/Field Names

1. **Check [`device.yaml`](device.yaml)** - All registers and fields are documented there
2. **Use IDE autocomplete** - Type `touch.ll.` to see all available registers
3. **Read a register** - Use `.read()` then autocomplete to see available field getters
4. **Write a register** - The closure parameter has autocomplete for all setters

## Scan Behavior

`scan()` performs a single 13-byte I2C read (registers `0x02`-`0x0E`) and returns `TouchData` containing:

- `touch_count`: number of active touch points (0-2)
- `points`: array of 2 `TouchPoint`s, each with:
  - `status`: `Touch` (new press), `Stream` (continued contact), or `Release`
  - `x`, `y`: 12-bit coordinates

The driver tracks touch state internally: the first scan detecting a finger reports `Touch`, subsequent scans report `Stream`, and when the finger lifts, `Release`.

## Register Map

The FT6336U register map is defined in [`device.yaml`](device.yaml), which `device-driver` uses to generate Rust code. This file specifies:

- Register names, addresses, and sizes
- Field names, bit positions, and access modes (Read-Only, Read-Write)
- Enumerations for field values (e.g., gesture IDs, power modes, touch events)
- Descriptions based on the datasheet

## Hardware Notes

- I2C address: `0x38` (available as `FT6336U_I2C_ADDRESS`)
- I2C speed: up to 400kHz
- Supply voltage: 2.8V-3.3V
- The driver does not manage reset or interrupt pins. Handle these in your application/BSP according to your board's wiring (GPIO, PMIC, I2C expander, etc.)
- Reset sequence: pull RST low for at least 5ms, release, wait at least 300ms before communicating

## Feature Flags

- **`default = []`**: No default features; async and blocking drivers are always available.
- **`std`**: Enables `std` features for `thiserror`.
- **`log`**: Enables `log` facade logging.
- **`defmt`**: Enables `defmt` logging and `defmt::Format` derives on all types.

## License

This project is dual-licensed under the [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE), at your option.
