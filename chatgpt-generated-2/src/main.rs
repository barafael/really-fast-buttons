#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::pac::Peripherals;
use stm32f4xx_hal::{
    gpio::{Edge, GpioExt},
    prelude::*,
    serial::Serial,
};

static INTERRUPT_COUNT: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

#[entry]
fn main() -> ! {
    let device = Peripherals::take().unwrap();

    let mut rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    let mut gpioa = device.GPIOA.split();
    let tx = gpioa.pa9.into_alternate_af7();
    let rx = gpioa.pa10.into_alternate_af7();

    let serial = Serial::usart1(device.USART1, (tx, rx), 115_200.bps(), clocks);
    let (mut tx, mut rx) = serial.split();

    let mut input_pins = [
        gpioa
            .pa0
            .into_pull_up_input()
            .into_interrupt_pin(&mut gpioa),
        gpioa
            .pa1
            .into_pull_up_input()
            .into_interrupt_pin(&mut gpioa),
        gpioa
            .pa2
            .into_pull_up_input()
            .into_interrupt_pin(&mut gpioa),
    ];

    for pin in input_pins.iter_mut() {
        pin.enable_interrupt(&mut gpioa, Edge::RisingEdge);
    }

    loop {
        if let Ok(byte) = rx.read() {
            match byte {
                b'r' => {
                    let count = INTERRUPT_COUNT.borrow(|count| *count.borrow());
                    let count_str = count.to_string();
                    for byte in count_str.as_bytes().iter() {
                        tx.write(*byte).unwrap();
                    }
                    tx.write(b'\r').unwrap();
                    tx.write(b'\n').unwrap();

                    INTERRUPT_COUNT.borrow(|count| *count.borrow_mut() = 0);
                }
                _ => {}
            }
        }
    }
}

#[interrupt]
fn EXTI0() {
    INTERRUPT_COUNT.borrow(|count| *count.borrow_mut() += 1);
}

#[interrupt]
fn EXTI1() {
    INTERRUPT_COUNT.borrow(|count| *count.borrow_mut() += 1);
}

#[interrupt]
fn EXTI2() {
    INTERRUPT_COUNT.borrow(|count| *count.borrow_mut() += 1);
}
