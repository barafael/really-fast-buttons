#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use defmt_rtt as _;
use nb::block;
use panic_probe as _;
use stm32f4xx_hal::{
    self as hal,
    gpio::{Alternate, Pin},
    interrupt,
    pac::{self, USART1},
    prelude::*,
    serial::{Event, Serial},
};

pub type Port = Serial<USART1, (Pin<'A', 9, Alternate<7>>, Pin<'A', 10, Alternate<7>>), u8>;

static mut SERIAL: Option<Port> = None;

static mut COUNTER: usize = 0;

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(_cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        defmt::println!("init");

        let rcc = dp.RCC.constrain();

        // Set up the system clock. We want to run at 48MHz for this one.
        let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(48.MHz()).freeze();
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
            // First, set instance
            let _ = SERIAL.insert(serial);
            // After that, enable USART1 interrupt
            NVIC::unmask(hal::interrupt::USART1);
        };

        let gpioc = dp.GPIOC.split();
        let _led = gpioc.pc13.into_push_pull_output();

        let pa0 = gpioa.pa0.internal_pull_up(true).into_input();
        let pa1 = gpioa.pa1.internal_pull_up(true).into_input();
        let pa2 = gpioa.pa2.internal_pull_up(true).into_input();

        let mut pa0_handled = false;
        let mut pa1_handled = false;
        let mut pa2_handled = false;

        loop {
            if pa0.is_high() {
                pa0_handled = false;
            } else if !pa0_handled {
                defmt::trace!("pa0 low");
                pa0_handled = true;
                unsafe {
                    COUNTER += 1;
                }
            }

            if pa1.is_high() {
                pa1_handled = false;
            } else if !pa1_handled {
                defmt::trace!("pa1 low");
                pa1_handled = true;
                unsafe {
                    COUNTER += 1;
                }
            }

            if pa2.is_high() {
                pa2_handled = false;
            } else if !pa2_handled {
                defmt::trace!("pa2 low");
                pa2_handled = true;
                unsafe {
                    COUNTER += 1;
                }
            }
        }
    }
    loop {}
}

#[interrupt]
fn USART1() {
    defmt::trace!("USART1 interrupt fired");
    unsafe {
        let serial = SERIAL.as_mut().unwrap();

        let byte = serial.read().unwrap();
        if byte == 0 {
            let bytes: [u8; 4] = COUNTER.to_le_bytes();
            block!(serial.write(1)).unwrap();
            block!(serial.write(bytes[0])).unwrap();
            block!(serial.write(bytes[1])).unwrap();
            block!(serial.write(bytes[2])).unwrap();
            block!(serial.write(bytes[3])).unwrap();
            block!(serial.write(0)).unwrap();
            block!(serial.write(0)).unwrap();
            block!(serial.write(0)).unwrap();
            block!(serial.write(0)).unwrap();
        }
        COUNTER = 0;
    };
}
