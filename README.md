# really-fast-buttons

Just a few very simple firmwares for STM32f411 which each count falling edges on three digital inputs (PA0, PA1, PA2).

The current edge count can be queried via serial on (tx/rx) = (PA9/PA10). It is reset after reading (destructive read).

## Approaches

* pac-blocking: busy-read the GPIOA IDR, checking if input bits are set (this exploits the fact that they are on a single port).
  Ironically this requires an at least interrupt-driven Serial port, so that the SERIAL RXNE doesn't also have to be polled.
* hal-blocking: busy-read the individual pins
  Ironically this requires an at least interrupt-driven Serial port, so that the SERIAL RXNE doesn't also have to be polled.
* raw-interrupts: three `#[interrupt]`s from `stm32f4xx-hal`, incrementing an atomic `u32`. Interrupt-driven serial port.
* mutex-cell: three `#[interrupt]`s from `stm32f4xx-hal` share access to a `core::cell::Cell` via a `cortex_m::interrupt::Mutex`.
* RTIC-atomic: three RTIC tasks with high priority which increment an atomic variable. Idle task manages Serial port.
* RTIC-shared: three RTIC tasks with high priority which increment a shared RTIC variable. Idle task manages Serial port.
* RTIC-counter-peripheral [WIP]: Uses TIM4 on ONE digital input.
* embassy-tasks: three embassy tasks which each `await` a level change, then increment an atomic variable.
* embassy-tasks-channels: waits in three tasks for any edge, then enqueues a `()` on a global channel. This channel is `select`ed upon together with the serial port in the main loop.

The interrupt-based approaches do benefit from the inputs being on separate interrupt lines 0, 1, and 2.
Otherwise, GPIO disambiguation would be required.

## Pathological Examples

These are plain wrong.

* embassy-select-pathological: three embassy futures which describe waiting on a level change, which are composed into a `select3`.
  On completion of one future, an atomic is incremented.
  This is not very responsive due to dropping the futures.
* raw-interrupts-pathological: Races galore hiding in the `unsafe` blocks.

## Why?

These are basic example programs for some of the major design approaches and frameworks in current embedded Rust.
Eventually, I'll benchmark up to which frequencies these edge counters hold, to see if there is any funny racy behaviour.
I may add some pathological versions too, which intentionally contain races.

## TODO

For fairness, ensure all implementations use:

* Same pin settings
* Same clock settings
* Same datatype for actual counting
* Same atomic orderings (which one applies best here?)