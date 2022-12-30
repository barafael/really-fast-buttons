#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use defmt_rtt as _;
use nb::block;
use panic_probe as _;
use rfb_proto::{SensorRequest, SensorResponse};
use stm32f4xx_hal::{
    self as hal,
    gpio::{Alternate, Edge, Pin},
    interrupt,
    pac::{self, EXTI, USART1},
    prelude::*,
    serial::{Event, Serial},
};

const ID: &str = env!("CARGO_PKG_NAME");

pub type Port = Serial<USART1, (Pin<'A', 9, Alternate<7>>, Pin<'A', 10, Alternate<7>>), u8>;

static mut SERIAL: Option<Port> = None;

static mut COUNTER: usize = 0;

#[entry]
fn main() -> ! {
    if let (Some(mut dp), Some(_cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        defmt::println!("init: {}", crate::ID);

        let rcc = dp.RCC.constrain();
        let mut syscfg = dp.SYSCFG.constrain();

        // Set up the system clock. We want to run at 16MHz for this one.
        let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(16.MHz()).freeze();
        let gpioa = dp.GPIOA.split();

        // Setup USART1
        let tx_pin = gpioa.pa9.into_alternate();
        let rx_pin = gpioa.pa10.into_alternate();

        // Configure serial
        let mut serial = dp
            .USART1
            .serial((tx_pin, rx_pin), 9600.bps(), &clocks)
            .unwrap();

        serial.listen(Event::Rxne);

        unsafe {
            // Enable USART1 interrupt
            NVIC::unmask(hal::interrupt::USART1);
            // Set global singleton
            let _ = SERIAL.insert(serial);
        };

        let gpioc = dp.GPIOC.split();
        let _led = gpioc.pc13.into_push_pull_output();

        let mut pa0 = gpioa.pa0.into_pull_up_input();
        pa0.make_interrupt_source(&mut syscfg);
        pa0.enable_interrupt(&mut dp.EXTI);
        pa0.trigger_on_edge(&mut dp.EXTI, Edge::Falling);

        let mut pa1 = gpioa.pa1.into_pull_up_input();
        pa1.make_interrupt_source(&mut syscfg);
        pa1.enable_interrupt(&mut dp.EXTI);
        pa1.trigger_on_edge(&mut dp.EXTI, Edge::Falling);

        let mut pa2 = gpioa.pa2.into_pull_up_input();
        pa2.make_interrupt_source(&mut syscfg);
        pa2.enable_interrupt(&mut dp.EXTI);
        pa2.trigger_on_edge(&mut dp.EXTI, Edge::Falling);

        unsafe {
            NVIC::unmask(hal::interrupt::EXTI0);
            NVIC::unmask(hal::interrupt::EXTI1);
            NVIC::unmask(hal::interrupt::EXTI2);
        }

        loop {
            continue;
        }
    }
    loop {}
}

#[interrupt]
fn USART1() {
    defmt::trace!("USART1 active");
    let serial = unsafe { SERIAL.as_mut().unwrap() };

    let byte = serial.read().unwrap();
    let request = rfb_proto::from_bytes(&[byte]);
    if request == Ok(SensorRequest::GetCount) {
        let count = unsafe {
            let count = COUNTER;
            COUNTER = 0;
            count
        };
        let response = SensorResponse::Count(count as u32);
        let bytes: rfb_proto::Vec<u8, 5> = rfb_proto::to_vec(&response).unwrap();
        for byte in bytes {
            block!(serial.write(byte)).unwrap();
        }
    }
}

#[interrupt]
fn EXTI0() {
    unsafe { (*EXTI::ptr()).pr.write(|w| w.bits(1 << 0)) };
    defmt::trace!("PA0 edge");
    unsafe {
        COUNTER += 1;
    }
}

#[interrupt]
fn EXTI1() {
    unsafe { (*EXTI::ptr()).pr.write(|w| w.bits(1 << 1)) };
    defmt::trace!("PA1 edge");
    unsafe {
        COUNTER += 1;
    }
}

#[interrupt]
fn EXTI2() {
    unsafe { (*EXTI::ptr()).pr.write(|w| w.bits(1 << 2)) };
    defmt::trace!("PA2 edge");
    unsafe {
        COUNTER += 1;
    }
}
