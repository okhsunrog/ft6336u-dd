# ft6336u-dd

`no_std` Rust driver for the FocalTech FT6336U capacitive touch controller, built on the [`device-driver`](https://crates.io/crates/device-driver) crate for type-safe register access.

Supports both blocking and async I2C via [`embedded-hal`](https://crates.io/crates/embedded-hal) 1.0 and [`embedded-hal-async`](https://crates.io/crates/embedded-hal-async) 1.0.

## Features

- Blocking (`Ft6336u`) and async (`Ft6336uAsync`) APIs from a single codebase using [`bisync`](https://crates.io/crates/bisync)
- Single I2C transaction `scan()` reads gesture + 2 touch points in 14 bytes
- 27 registers covering touch data, configuration, gestures, power management, and chip identification
- Type-safe enums for device mode, gesture ID, touch events, power mode, control mode, and gesture/interrupt mode
- Optional `defmt` support

## Usage

### Blocking

```rust
use ft6336u_dd::{Ft6336u, TouchStatus};

let mut touch = Ft6336u::new(i2c);

let data = touch.scan().unwrap();
for point in &data.points {
    if point.status != TouchStatus::Release {
        // Handle touch at (point.x, point.y)
    }
}
```

### Async

```rust
use ft6336u_dd::{Ft6336uAsync, TouchStatus};

let mut touch = Ft6336uAsync::new(i2c);

let data = touch.scan().await.unwrap();
for point in &data.points {
    if point.status != TouchStatus::Release {
        // Handle touch at (point.x, point.y)
    }
}
```

### Gesture detection

```rust
use ft6336u_dd::GestureId;

let data = touch.scan().unwrap();
match data.gesture {
    GestureId::MoveUp => { /* swipe up */ }
    GestureId::MoveDown => { /* swipe down */ }
    GestureId::MoveLeft => { /* swipe left */ }
    GestureId::MoveRight => { /* swipe right */ }
    GestureId::ZoomIn => { /* pinch out */ }
    GestureId::ZoomOut => { /* pinch in */ }
    _ => {}
}
```

### Configuration

```rust
use ft6336u_dd::PowerModeEnum;

// Set touch sensitivity (lower = more sensitive)
touch.write_touch_threshold(40).unwrap();

// Set report rate in active mode (Hz)
touch.write_active_rate(60).unwrap();

// Enter hibernate mode
touch.write_power_mode(PowerModeEnum::Hibernate).unwrap();
```

### Low-level register access

The `ll` field exposes the full device-driver generated register API:

```rust
let mut op = touch.ll.chip_id();
let chip_id = op.read().unwrap();
assert_eq!(chip_id.value(), 0x64); // FT6336U
```

## Scan behavior

`scan()` performs a single 14-byte I2C read (registers `0x01`-`0x0E`) and returns `TouchData` containing:

- `gesture`: detected gesture (`GestureId` enum)
- `touch_count`: number of active touch points (0-2)
- `points`: array of 2 `TouchPoint`s, each with:
  - `status`: `Touch` (new press), `Stream` (continued contact), or `Release`
  - `x`, `y`: 12-bit coordinates

The driver tracks touch state internally: the first scan detecting a finger reports `Touch`, subsequent scans report `Stream`, and when the finger lifts, `Release`.

## Register map

See [`device.yaml`](device.yaml) for the full register definition (27 registers with addresses, field layouts, and enum conversions).

## Cargo features

| Feature | Description |
|---------|-------------|
| `defmt` | Enable `defmt::Format` derives on all types |
| `log` | Enable `log` crate integration |
| `std` | Enable `std` error trait support |

## Hardware notes

- I2C address: `0x38` (available as `FT6336U_I2C_ADDRESS`)
- I2C speed: up to 400kHz
- Supply voltage: 2.8V-3.3V
- The driver does not manage reset or interrupt pins. Handle these in your application/BSP according to your board's wiring (GPIO, PMIC, I2C expander, etc.)
- Reset sequence: pull RST low for at least 5ms, release, wait at least 300ms before communicating

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
