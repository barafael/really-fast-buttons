#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::sync::atomic::{AtomicU32, Ordering};

use embassy_executor::Spawner;
use embassy_futures::select::Either3;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pull};
use {defmt_rtt as _, panic_probe as _};

static COUNTER: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let pa0 = Input::new(p.PA0, Pull::Up);
    let pa1 = Input::new(p.PA1, Pull::Up);
    let pa2 = Input::new(p.PA2, Pull::Up);

    let mut pa0 = ExtiInput::new(pa0, p.EXTI0);
    let mut pa1 = ExtiInput::new(pa1, p.EXTI1);
    let mut pa2 = ExtiInput::new(pa2, p.EXTI2);

    loop {
        // TODO won't this design result in a brief moment of non-reactivity?
        // Try a task-based design with actual concurrency.
        let zero = pa0.wait_for_rising_edge();
        let one = pa1.wait_for_rising_edge();
        let two = pa2.wait_for_rising_edge();

        let f = embassy_futures::select::select3(zero, one, two).await;
        match f {
            Either3::First(_) => {
                COUNTER.fetch_add(1, Ordering::Relaxed);
            }
            Either3::Second(_) => {
                COUNTER.fetch_add(1, Ordering::Relaxed);
            }
            Either3::Third(_) => {
                COUNTER.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}
