#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::sync::atomic::{AtomicU32, Ordering};

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pull};
use {defmt_rtt as _, panic_probe as _};

static COUNTER: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let pa0 = Input::new(p.PA0, Pull::Up);
    let mut pa0 = ExtiInput::new(pa0, p.EXTI0);

    loop {
        pa0.wait_for_rising_edge().await;
        COUNTER.fetch_add(1, Ordering::Relaxed);
        defmt::trace!("pa0 triggered");
    }
}
