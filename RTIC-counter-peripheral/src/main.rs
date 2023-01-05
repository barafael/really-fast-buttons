#![deny(unsafe_code)]
#![no_main]
#![no_std]

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
    use rfb_proto::{SensorRequest, SensorResponse};
    use stm32f4xx_hal::{
        block,
        gpio::{Edge, ExtiPin, GpioExt, PinState},
        pac::{TIM4, USART1},
        prelude::*,
        serial::{Rx, Tx},
        timer::Timer,
    };
    use systick_monotonic::Systick;

    const CLOCK_FREQ_HZ: u32 = 32_000_000;

    #[monotonic(binds = SysTick, default = true)]
    type Tonic = Systick<1000>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        timer: TIM4,
        tx: Tx<USART1>,
        rx: Rx<USART1>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // System clock and monotonic timer
        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(CLOCK_FREQ_HZ.Hz()).freeze();

        // GPIO pins
        let gpioc = ctx.device.GPIOC.split();
        let _led = gpioc.pc13.into_push_pull_output_in_state(PinState::High);

        let gpiob = ctx.device.GPIOB.split();
        let mut input = gpiob.pb8.into_pull_down_input();

        // Configure TIM4 as a hardware counter for CLK edges using TI2 input.
        // Register configuration per ST RM0383, section 13.3.3.
        // Use the HAL to enable and reset, then release for manual register config.
        let timer = Timer::new(ctx.device.TIM4, &clocks).release();
        timer
            .ccmr1_input()
            .write(|w| w.cc2s().ti2().ic2f().bits(0b0011));
        timer.ccer.write(|w| w.cc2np().set_bit().cc2p().set_bit());
        timer.smcr.write(|w| w.sms().ext_clock_mode().ts().ti2fp2());
        timer.cr1.write(|w| w.cen().set_bit());

        // Enable edge-triggered interrupt for input pin
        input.make_interrupt_source(&mut ctx.device.SYSCFG.constrain());
        input.enable_interrupt(&mut ctx.device.EXTI);
        input.trigger_on_edge(&mut ctx.device.EXTI, Edge::RisingFalling);

        let gpioa = ctx.device.GPIOA.split();
        let tx_pin = gpioa.pa9.into_alternate();
        let rx_pin = gpioa.pa10.into_alternate();

        // Configure serial
        let serial = ctx
            .device
            .USART1
            .serial((tx_pin, rx_pin), 9600.bps(), &clocks)
            .unwrap();

        let (tx, rx) = serial.split();

        let mono = Systick::new(ctx.core.SYST, CLOCK_FREQ_HZ);

        (Shared {}, Local { timer, tx, rx }, init::Monotonics(mono))
    }

    #[idle(local = [tx, rx, timer])]
    fn idle(ctx: idle::Context) -> ! {
        loop {
            let byte = block!(ctx.local.rx.read()).unwrap();
            defmt::trace!("USART1 active");
            let request = rfb_proto::from_bytes(&[byte]);
            match request {
                Ok(SensorRequest::GetCount) => {
                    let clk_count: u16 = ctx.local.timer.cnt.read().cnt().bits();
                    defmt::info!("Count: {}", clk_count);
                    let response = SensorResponse::Count(clk_count as u32);
                    let bytes: rfb_proto::Vec<u8, 5> = rfb_proto::to_vec(&response).unwrap();
                    for byte in bytes {
                        block!(ctx.local.tx.write(byte)).unwrap();
                    }
                }
                Ok(SensorRequest::WhoAreYou) => {
                    defmt::warn!("WhoAreYou not implemented yet")
                }
                Err(e) => defmt::error!("Not a request {}", e),
            }
        }
    }
}
