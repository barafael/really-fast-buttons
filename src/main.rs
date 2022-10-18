#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::sync::atomic::{AtomicU32, Ordering};

use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::peripherals::{PA0, PA1, PA2};
use {defmt_rtt as _, panic_probe as _};

static COUNTER: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let pa0 = Input::new(p.PA0, Pull::Up);
    let pa1 = Input::new(p.PA1, Pull::Up);
    let pa2 = Input::new(p.PA2, Pull::Up);

    let pa0 = ExtiInput::new(pa0, p.EXTI0);
    let pa1 = ExtiInput::new(pa1, p.EXTI1);
    let pa2 = ExtiInput::new(pa2, p.EXTI2);

    let pa0_hdl = monitor_pa0(pa0);
    let pa1_hdl = monitor_pa1(pa1);
    let pa2_hdl = monitor_pa2(pa2);

    spawner.spawn(pa0_hdl).unwrap();
    spawner.spawn(pa1_hdl).unwrap();
    spawner.spawn(pa2_hdl).unwrap();
}

#[embassy_executor::task]
async fn monitor_pa0(mut pa0: ExtiInput<'static, PA0>) {
    loop {
        pa0.wait_for_any_edge().await;
        COUNTER.fetch_add(1, Ordering::Acquire);
        defmt::println!("{}", COUNTER.load(Ordering::SeqCst));
    }
}

#[embassy_executor::task]
async fn monitor_pa1(mut pa1: ExtiInput<'static, PA1>) {
    loop {
        pa1.wait_for_any_edge().await;
        COUNTER.fetch_add(1, Ordering::Acquire);
    }
}

#[embassy_executor::task]
async fn monitor_pa2(mut pa2: ExtiInput<'static, PA2>) {
    loop {
        pa2.wait_for_any_edge().await;
        COUNTER.fetch_add(1, Ordering::Acquire);
    }
}
