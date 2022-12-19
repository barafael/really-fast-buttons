# RFB using `Mutex<Cell<u32>>`

Three interrupt handlers for digital inputs + one interrupt handler for serial port communication share access to a `Mutex<Cell<u32>>`.
Uses a `cortex_m::interrupt::Mutex` and a `core::cell::Cell`.
