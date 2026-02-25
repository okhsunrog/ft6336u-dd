# ft6336u-dd

`no_std` FT6336U capacitive touch controller driver based on `device-driver` + `bisync`.

This crate provides both blocking (`Ft6336u`) and async (`Ft6336uAsync`) APIs from one codebase,
and exposes a generated low-level API via `driver.ll`.
