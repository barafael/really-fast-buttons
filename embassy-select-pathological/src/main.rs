#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use panic_probe as _;

use core::sync::atomic::{AtomicU32, Ordering};
use embassy_executor::Spawner;
use embassy_futures::select::Either3;
use embassy_stm32::{
    dma::NoDma,
    exti::ExtiInput,
    gpio::{Input, Pull},
    interrupt, peripherals,
    usart::{BufferedUart, Config, State, Uart},
};
use embedded_io::asynch::{BufRead, Write};
use rfb_proto::{SensorRequest, SensorResponse};

static COUNTER: AtomicU32 = AtomicU32::new(0);

const ID: &str = env!("CARGO_PKG_NAME");

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::println!("init: {}", crate::ID);
    let p = embassy_stm32::init(embassy_stm32::Config::default());

    let usart = p.USART1;
    let pa10 = p.PA10;
    let pa9 = p.PA9;

    let pa0 = Input::new(p.PA0, Pull::Up);
    let pa1 = Input::new(p.PA1, Pull::Up);
    let pa2 = Input::new(p.PA2, Pull::Up);

    let mut pa0 = ExtiInput::new(pa0, p.EXTI0);
    let mut pa1 = ExtiInput::new(pa1, p.EXTI1);
    let mut pa2 = ExtiInput::new(pa2, p.EXTI2);

    spawner.spawn(monitor_usart(usart, pa10, pa9)).unwrap();

    loop {
        // TODO won't this design result in a brief moment of non-reactivity?
        // Try a task-based design with actual concurrency.
        let zero = pa0.wait_for_rising_edge();
        let one = pa1.wait_for_rising_edge();
        let two = pa2.wait_for_rising_edge();

        let f = embassy_futures::select::select3(zero, one, two);
        match f.await {
            Either3::First(_) => {
                defmt::trace!("PA0 edge");
            }
            Either3::Second(_) => {
                defmt::trace!("PA1 edge");
            }
            Either3::Third(_) => {
                defmt::trace!("PA2 edge");
            }
        }
        COUNTER.fetch_add(1, Ordering::Relaxed);
    }
}

#[embassy_executor::task]
async fn monitor_usart(usart: peripherals::USART1, pa10: peripherals::PA10, pa9: peripherals::PA9) {
    let mut config = Config::default();
    config.baudrate = 9600;
    let usart = Uart::new(usart, pa10, pa9, NoDma, NoDma, config);

    let mut state = State::new();
    let irq = interrupt::take!(USART1);
    let mut tx_buf = [0u8; 32];
    let mut rx_buf = [0u8; 1];
    let mut buf_usart = BufferedUart::new(&mut state, usart, irq, &mut tx_buf, &mut rx_buf);
    let (mut rx, mut tx) = buf_usart.split();
    loop {
        let buf = rx.fill_buf().await.unwrap();
        defmt::trace!("USART1 active");
        let n = buf.len();
        let byte = rfb_proto::from_bytes(buf);
        rx.consume(n);
        if byte == Ok(SensorRequest::GetCount) {
            let count = COUNTER.swap(0, Ordering::Acquire);
            let response = SensorResponse::Count(count as u32);
            let bytes: rfb_proto::Vec<u8, 5> = rfb_proto::to_vec(&response).unwrap();
            tx.write_all(&bytes).await.unwrap();
            tx.flush().await.unwrap();
        }
    }
}
