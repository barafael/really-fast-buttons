#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

use core::alloc::Layout;
use cortex_m;
use cortex_m::asm;
use cortex_m_rt::exception;
use cortex_m_rt::{entry, ExceptionFrame};
use embedded_hal::digital::v2::OutputPin;
use freertos_rust::*;
use hal::pac::Peripherals;
use hal::rcc::RccExt;
use stm32f4xx_hal::{self as hal, gpio::*, prelude::*};

use defmt_rtt as _;
use panic_probe as _;

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;

fn delay() {
    let mut _i = 0;
    for _ in 0..2_00 {
        _i += 1;
    }
}

fn delay_n(n: i32) {
    for _ in 0..n {
        delay();
    }
}

pub struct MyDevice<D1: OutputPin> {
    d1: D1,
}

impl<D1: OutputPin> MyDevice<D1> {
    pub fn from_pins(d1: D1) -> MyDevice<D1> {
        MyDevice { d1 }
    }
    pub fn set_led(&mut self, on: bool) {
        if on {
            self.d1.set_high();
        } else {
            self.d1.set_low();
        }
    }
}

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();

    // Set up the system clock. We want to run at 48MHz for this one.
    let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(24.MHz()).freeze();

    let gpioc = dp.GPIOC.split();
    let mut device = MyDevice::from_pins(gpioc.pc13.into_push_pull_output());
    device.set_led(false);
    Task::new()
        .name("hello")
        .stack_size(256)
        .priority(TaskPriority(2))
        .start(move |_| loop {
            freertos_rust::CurrentTask::delay(Duration::ms(1000));
            device.set_led(true);
            freertos_rust::CurrentTask::delay(Duration::ms(1000));
            device.set_led(false);
            defmt::println!("blink");
        })
        .unwrap();
    FreeRtosUtils::start_scheduler();
}

#[exception]
unsafe fn DefaultHandler(_irqn: i16) {
    // custom default handler
    // irqn is negative for Cortex-M exceptions
    // irqn is positive for device specific (line IRQ)
    // set_led(true);(true);
    // panic!("Exception: {}", irqn);
}

#[exception]
unsafe fn HardFault(_ef: &ExceptionFrame) -> ! {
    // Blink 3 times long when exception occures
    delay_n(10);
    for _ in 0..3 {
        // set_led(true);
        // delay_n(1000);
        // set_led(false);
        // delay_n(555);
    }
    loop {}
}

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    //set_led(true);
    asm::bkpt();
    loop {}
}

#[no_mangle]
fn vApplicationStackOverflowHook(pxTask: FreeRtosTaskHandle, pcTaskName: FreeRtosCharPtr) {
    asm::bkpt();
}
