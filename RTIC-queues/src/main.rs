#![no_main]
#![no_std]
#![allow(unused_imports)]

use panic_probe as _;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

use defmt_rtt as _;
use rtic::app;

const ID: &str = env!("CARGO_PKG_NAME");

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use core::fmt::Write;
    use core::sync::atomic::{AtomicUsize, Ordering};
    use rfb_proto::{SensorRequest, SensorResponse};
    use stm32f4xx_hal::{
        block,
        gpio::{gpioa::PA0, gpioc::PC6, Alternate, Edge, Input, Output, Pin, PushPull},
        pac::USART1,
        prelude::*,
        serial::{Rx, Serial, Tx},
    };
    use systick_monotonic::{fugit::Duration, Systick};

    static mut COUNTER: u32 = 0;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        pa0: Pin<'A', 0, Input>,
        pa1: Pin<'A', 1, Input>,
        pa2: Pin<'A', 2, Input>,
        tx: Tx<USART1>,
        rx: Rx<USART1>,
    }

    #[monotonic(binds = SysTick, default = true)]
    type Tonic = Systick<1000>;

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::println!("init: {}", crate::ID);

        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        let gpioc = ctx.device.GPIOC.split();
        let _led = gpioc.pc13.into_push_pull_output();

        let mut sys_cfg = ctx.device.SYSCFG.constrain();

        let gpioa = ctx.device.GPIOA.split();

        let mut pa0 = gpioa.pa0.into_pull_up_input();
        let mut pa1 = gpioa.pa1.into_pull_up_input();
        let mut pa2 = gpioa.pa2.into_pull_up_input();

        pa0.make_interrupt_source(&mut sys_cfg);
        pa1.make_interrupt_source(&mut sys_cfg);
        pa2.make_interrupt_source(&mut sys_cfg);

        pa0.enable_interrupt(&mut ctx.device.EXTI);
        pa1.enable_interrupt(&mut ctx.device.EXTI);
        pa2.enable_interrupt(&mut ctx.device.EXTI);

        pa0.trigger_on_edge(&mut ctx.device.EXTI, Edge::Falling);
        pa1.trigger_on_edge(&mut ctx.device.EXTI, Edge::Falling);
        pa2.trigger_on_edge(&mut ctx.device.EXTI, Edge::Falling);

        // Setup USART1
        let tx_pin = gpioa.pa9.into_alternate();
        let rx_pin = gpioa.pa10.into_alternate();

        // Configure serial
        let serial = ctx
            .device
            .USART1
            .serial((tx_pin, rx_pin), 9600.bps(), &clocks)
            .unwrap();

        let (tx, rx) = serial.split();

        let mono = Systick::new(ctx.core.SYST, 48_000_000);

        (
            Shared {},
            Local {
                pa0,
                pa1,
                pa2,
                tx,
                rx,
            },
            init::Monotonics(mono),
        )
    }

    #[idle(local = [tx, rx])]
    fn idle(ctx: idle::Context) -> ! {
        loop {
            let byte = block!(ctx.local.rx.read()).unwrap();
            defmt::trace!("USART1 active");
            let byte = rfb_proto::from_bytes(&[byte]);
            if byte == Ok(SensorRequest::GetCount) {
                let count = unsafe {
                    let count = COUNTER;
                    COUNTER = 0;
                    count
                };
                let response = SensorResponse::Count(count);
                let bytes: rfb_proto::Vec<u8, 5> = rfb_proto::to_vec(&response).unwrap();
                for byte in bytes {
                    block!(ctx.local.tx.write(byte)).unwrap();
                }
            }
        }
    }

    // TODO adapt capacity here
    #[task(capacity = 32, priority = 1)]
    fn increment(_c: increment::Context) {
        unsafe {
            COUNTER += 1;
        }
    }

    #[task(binds = EXTI0, local = [pa0], priority = 3)]
    fn on_exti0(ctx: on_exti0::Context) {
        ctx.local.pa0.clear_interrupt_pending_bit();
        defmt::trace!("PA0 edge");
        if let Err(()) = increment::spawn() {
            defmt::trace!("dropped event");
        }
    }

    #[task(binds = EXTI1, local = [pa1], priority = 3)]
    fn on_exti1(ctx: on_exti1::Context) {
        ctx.local.pa1.clear_interrupt_pending_bit();
        defmt::trace!("PA1 edge");
        if let Err(()) = increment::spawn() {
            defmt::trace!("dropped event");
        }
    }

    #[task(binds = EXTI2, local = [pa2], priority = 3)]
    fn on_exti2(ctx: on_exti2::Context) {
        ctx.local.pa2.clear_interrupt_pending_bit();
        defmt::trace!("PA2 edge");
        if let Err(()) = increment::spawn() {
            defmt::trace!("dropped event");
        }
    }
}
