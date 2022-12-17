#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

use core::{
    alloc::Layout,
    sync::atomic::{AtomicUsize, Ordering},
};
use cortex_m::{self, asm};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use freertos_rust::*;
use hal::{
    block, nb,
    pac::{Peripherals, USART1},
    prelude::*,
    rcc::RccExt,
    serial::Serial,
};
use rfb_proto::{SensorRequest, SensorResponse};
use stm32f4xx_hal::{self as hal, gpio::*};

use defmt_rtt as _;
use panic_probe as _;

pub type Port = Serial<USART1, (Pin<'A', 9, Alternate<7>>, Pin<'A', 10, Alternate<7>>), u8>;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

const ID: &str = env!("CARGO_PKG_NAME");

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

pub struct Blinker<D1: OutputPin> {
    d1: D1,
}

impl<D1: OutputPin> Blinker<D1>
where
    D1::Error: core::fmt::Debug,
{
    pub fn from_pins(d1: D1) -> Blinker<D1> {
        Blinker { d1 }
    }

    pub fn set_led(&mut self, on: bool) {
        if on {
            self.d1.set_high().unwrap();
        } else {
            self.d1.set_low().unwrap();
        }
    }
}

pub struct Monitor<D1: InputPin> {
    d1: D1,
    last_state: bool,
    id: &'static str,
}

impl<D1> Monitor<D1>
where
    D1: InputPin,
    D1::Error: core::fmt::Debug,
{
    pub fn from_pins(d1: D1, id: &'static str) -> Monitor<D1> {
        let last_state = d1.is_high().unwrap();
        Monitor { d1, last_state, id }
    }

    pub fn run(&mut self) -> ! {
        loop {
            if self.d1.is_high().unwrap() {
                self.last_state = false;
            } else if !self.last_state {
                defmt::trace!("{} edge", self.id);
                self.last_state = true;
                COUNTER.fetch_add(1, Ordering::SeqCst);
            }
        }
    }
}

pub struct Reporter {
    serial: Port,
}

impl Reporter {
    pub fn from_port(port: Port) -> Self {
        Self { serial: port }
    }

    pub fn run(&mut self) -> ! {
        loop {
            let byte = loop {
                match self.serial.read() {
                    Ok(byte) => break byte,
                    Err(nb::Error::WouldBlock) => continue,
                    Err(e) => panic!("{:?}", e),
                }
            };
            defmt::trace!("{:x}", byte);
            defmt::trace!("USART1 active");
            let byte = rfb_proto::from_bytes(&[byte]);
            if let Ok(SensorRequest::GetCount) = byte {
                let count = COUNTER.swap(0, Ordering::Acquire);
                let response = SensorResponse::Count(count as u32);
                let bytes: rfb_proto::Vec<u8, 5> = rfb_proto::to_vec(&response).unwrap();
                for byte in bytes {
                    block!(self.serial.write(byte)).unwrap();
                }
            }
        }
    }
}

#[entry]
fn main() -> ! {
    defmt::println!("init: {}", crate::ID);

    let dp = Peripherals::take().unwrap();

    let gpioc = dp.GPIOC.split();
    let mut blinker = Blinker::from_pins(gpioc.pc13.into_push_pull_output());
    blinker.set_led(false);

    let gpioa = dp.GPIOA.split();
    let pa0 = gpioa.pa0.internal_pull_up(true).into_input();
    let pa1 = gpioa.pa1.internal_pull_up(true).into_input();
    let pa2 = gpioa.pa2.internal_pull_up(true).into_input();

    let mut monitor0 = Monitor::from_pins(pa0, "PA0");
    let mut monitor1 = Monitor::from_pins(pa1, "PA1");
    let mut monitor2 = Monitor::from_pins(pa2, "PA2");

    let rcc = dp.RCC.constrain();

    // Set up the system clock. We want to run at 48MHz for this one.
    let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(48.MHz()).freeze();

    // Setup USART1
    let tx_pin = gpioa.pa9.into_alternate();
    let rx_pin = gpioa.pa10.into_alternate();

    // Configure serial
    let serial: Port = dp
        .USART1
        .serial((tx_pin, rx_pin), 9600.bps(), &clocks)
        .unwrap();
    let mut reporter = Reporter::from_port(serial);

    Task::new()
        .name("blink")
        .stack_size(256)
        .priority(TaskPriority(2))
        .start(move |_| loop {
            freertos_rust::CurrentTask::delay(Duration::ms(1000));
            blinker.set_led(true);
            freertos_rust::CurrentTask::delay(Duration::ms(1000));
            blinker.set_led(false);
            defmt::println!("blink");
        })
        .unwrap();

    Task::new()
        .name("monitor_pa0")
        .stack_size(256)
        .priority(TaskPriority(1))
        .start(move |_| monitor0.run())
        .unwrap();

    Task::new()
        .name("monitor_pa1")
        .stack_size(256)
        .priority(TaskPriority(1))
        .start(move |_| monitor1.run())
        .unwrap();

    Task::new()
        .name("monitor_pa2")
        .stack_size(256)
        .priority(TaskPriority(1))
        .start(move |_| monitor2.run())
        .unwrap();

    Task::new()
        .name("report")
        .stack_size(512)
        .priority(TaskPriority(3))
        .start(move |_| reporter.run())
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
