#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use defmt::println;
use stm32f4::stm32f407;
use stm32f4::stm32f407::interrupt;

static mut COUNT: u32 = 0;

fn main() {
    let peripherals = stm32f407::Peripherals::take().unwrap();

    let gpioa = &peripherals.GPIOA;
    let rcc = &peripherals.RCC;
    let usart2 = &peripherals.USART2;

    // Enable clock for GPIOA and USART2
    rcc.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
    rcc.apb1enr.modify(|_, w| w.usart2en().set_bit());

    // Configure PA0, PA1, and PA2 as inputs with pull-up resistors
    gpioa
        .moder
        .modify(|_, w| w.moder0().input().moder1().input().moder2().input());
    gpioa
        .pupdr
        .modify(|_, w| w.pupdr0().pull_up().pupdr1().pull_up().pupdr2().pull_up());

    // Configure PA0, PA1, and PA2 to trigger an interrupt on rising edges
    gpioa
        .exticr1
        .modify(|_, w| w.exti0().bits(0).exti1().bits(0).exti2().bits(0));
    gpioa
        .rise
        .modify(|_, w| w.rise0().set_bit().rise1().set_bit().rise2().set_bit());
    gpioa
        .imr
        .modify(|_, w| w.im0().set_bit().im1().set_bit().im2().set_bit());

    // Configure USART2 for 115200 baud, 8N1
    usart2.brr.write(|w| w.brr().bits(0x683)); // 115200 baud
    usart2
        .cr1
        .modify(|_, w| w.ue().set_bit().re().set_bit().te().set_bit()); // Enable USART, RX, and TX

    // Enable USART2 interrupt in NVIC
    unsafe {
        stm32f407::NVIC::unmask(interrupt::USART2);
    }

    loop {
        // Poll for incoming characters on serial port
        if usart2.isr.read().rxne().bit_is_set() {
            let c = usart2.rdr.read().rdr().bits() as u8 as char;

            if c == 'r' {
                let count = unsafe { COUNT };
                println!("{}", count);
                unsafe { COUNT = 0 };
            }
        }
    }
}

#[interrupt]
fn EXTI0() {
    unsafe {
        COUNT += 1;
    }
    stm32f407::EXTI::unpend(0);
}

#[interrupt]
fn EXTI1() {
    unsafe {
        COUNT += 1;
    }
    stm32f407::EXTI::unpend(1);
}

#[interrupt]
fn EXTI3() {
    unsafe {
        COUNT += 1;
    }
    stm32f407::EXTI::unpend(3);
}

#[interrupt]
fn USART2() {
    let usart2 = USART2::ptr();

    // Check if the interrupt was triggered by a received character
    if usart2.isr.read().rxne().bit_is_set() {
        let c = usart2.rdr.read().rdr().bits() as u8 as char;

        if c == 'r' {
            let count = unsafe { COUNT };
            println!("{}", count);
            unsafe { COUNT = 0 };
        }
    }
}
