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
    use stm32f4xx_hal::{
        gpio::{Edge, ExtiPin, GpioExt, Input, Output, Pin, PinState, PushPull},
        pac::TIM4,
        prelude::*,
        timer::Timer,
    };
    use systick_monotonic::Systick;

    const CLOCK_FREQ_HZ: u32 = 16_000_000;

    #[monotonic(binds = SysTick, default = true)]
    type Tonic = Systick<1000>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Pin<'C', 13, Output<PushPull>>,
        input: Pin<'B', 8, Input>,
        timer: TIM4,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // System clock and monotonic timer
        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(CLOCK_FREQ_HZ.Hz()).freeze();

        // GPIO pins
        let gpioc = ctx.device.GPIOC.split();
        let led = gpioc.pc13.into_push_pull_output_in_state(PinState::High);

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

        let mono = Systick::new(ctx.core.SYST, CLOCK_FREQ_HZ);

        (
            Shared {},
            Local { led, input, timer },
            init::Monotonics(mono),
        )
    }

    #[task(binds = EXTI9_5, local = [input, timer, led])]
    fn on_exti(ctx: on_exti::Context) {
        ctx.local.input.clear_interrupt_pending_bit();
        let clk_count: u16 = ctx.local.timer.cnt.read().cnt().bits();
        defmt::info!("Count: {}", clk_count);
        ctx.local.led.toggle();
    }
}
