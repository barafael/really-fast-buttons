#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use defmt_rtt as _;
use nb::block;
use panic_probe as _;
use rfb_proto::{SensorRequest, SensorResponse};
use stm32f4xx_hal::{
    self as hal,
    gpio::{Alternate, Pin},
    interrupt,
    pac::{self, GPIOA, USART1},
    prelude::*,
    serial::{Event, Serial},
};

pub type Port = Serial<USART1, (Pin<'A', 9, Alternate<7>>, Pin<'A', 10, Alternate<7>>), u8>;

static mut SERIAL: Option<Port> = None;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

const ID: &str = env!("CARGO_PKG_NAME");

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(_cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        defmt::println!("init: {}", crate::ID);

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

        let _ = gpioa.pa0.internal_pull_up(true).into_input();
        let _ = gpioa.pa1.internal_pull_up(true).into_input();
        let _ = gpioa.pa2.internal_pull_up(true).into_input();

        let mut pa0_handled = false;
        let mut pa1_handled = false;
        let mut pa2_handled = false;

        loop {
            let idr_a = unsafe { (*GPIOA::ptr()).idr.read() }.bits();
            if idr_a & 0b1 != 0 {
                pa0_handled = false;
            } else if !pa0_handled {
                defmt::trace!("PA0 edge");
                pa0_handled = true;
                COUNTER.fetch_add(1, Ordering::SeqCst);
            }

            if idr_a & 0b10 != 0 {
                pa1_handled = false;
            } else if !pa1_handled {
                defmt::trace!("PA1 edge");
                pa1_handled = true;
                COUNTER.fetch_add(1, Ordering::SeqCst);
            }

            if idr_a & 0b100 != 0 {
                pa2_handled = false;
            } else if !pa2_handled {
                defmt::trace!("PA2 edge");
                pa2_handled = true;
                COUNTER.fetch_add(1, Ordering::SeqCst);
            }
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
    if let Ok(SensorRequest::GetCount) = request {
        let count = COUNTER.swap(0, Ordering::SeqCst);
        let response = SensorResponse::Count(count as u32);
        let bytes: rfb_proto::Vec<u8, 5> = rfb_proto::to_vec(&response).unwrap();
        for byte in bytes {
            block!(serial.write(byte)).unwrap();
        }
    }
}
