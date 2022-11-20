#![deny(unsafe_code)]
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

#[app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use core::fmt::Write;
    use core::sync::atomic::{AtomicUsize, Ordering};
    use rfb_proto::SensorMessage;
    use stm32f4xx_hal::{
        block,
        gpio::{gpioa::PA0, gpioc::PC6, Alternate, Edge, Input, Output, Pin, PushPull},
        pac::USART1,
        prelude::*,
        serial::{Rx, Serial, Tx},
    };
    use systick_monotonic::{fugit::Duration, Systick};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

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
        defmt::println!("init");

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

        // configure serial
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
        // Consider using USB peripheral instead
        // https://github.com/stm32-rs/stm32f4xx-hal/blob/master/examples/usb_serial_poll.rs
        // or
        // https://github.com/stm32-rs/stm32f4xx-hal/blob/master/examples/usb_serial_irq.rs
        loop {
            let byte = block!(ctx.local.rx.read()).unwrap();
            let byte = rfb_proto::from_bytes(&[byte]);
            if let Ok(SensorMessage::Request) = byte {
                let count = COUNTER.swap(0, Ordering::Acquire);
                let response = SensorMessage::Response(count as u64);
                let bytes: rfb_proto::Vec<u8, 9> = rfb_proto::to_vec(&response).unwrap();
                for byte in bytes {
                    block!(ctx.local.tx.write(byte)).unwrap();
                }
            }
        }
    }

    #[task(binds = EXTI0, local = [pa0])]
    fn on_exti0(ctx: on_exti0::Context) {
        ctx.local.pa0.clear_interrupt_pending_bit();
        defmt::trace!("pa0 triggered");
        COUNTER.fetch_add(1, Ordering::SeqCst);
    }

    #[task(binds = EXTI1, local = [pa1])]
    fn on_exti1(ctx: on_exti1::Context) {
        ctx.local.pa1.clear_interrupt_pending_bit();
        defmt::trace!("pa1 triggered");
        COUNTER.fetch_add(1, Ordering::SeqCst);
    }

    #[task(binds = EXTI2, local = [pa2])]
    fn on_exti2(ctx: on_exti2::Context) {
        ctx.local.pa2.clear_interrupt_pending_bit();
        defmt::trace!("pa2 triggered");
        COUNTER.fetch_add(1, Ordering::SeqCst);
    }
}
