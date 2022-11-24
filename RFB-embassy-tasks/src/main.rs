#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::sync::atomic::{AtomicU32, Ordering};
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::interrupt::{self};
use embassy_stm32::peripherals::{PA0, PA1, PA2};
use embassy_stm32::usart::{BufferedUart, Config, State, Uart};
use embedded_io::asynch::{BufRead, Write};
use rfb_proto::SensorMessage;
use {defmt_rtt as _, panic_probe as _};

static COUNTER: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut config = Config::default();
    config.baudrate = 9600;
    let usart = Uart::new(p.USART1, p.PA10, p.PA9, NoDma, NoDma, config);

    let mut state = State::new();
    let irq = interrupt::take!(USART1);
    let mut tx_buf = [0u8; 32];
    let mut rx_buf = [0u8; 1];
    let mut buf_usart = BufferedUart::new(&mut state, usart, irq, &mut tx_buf, &mut rx_buf);

    let pa0 = Input::new(p.PA0, Pull::Up);
    let pa1 = Input::new(p.PA1, Pull::Up);
    let pa2 = Input::new(p.PA2, Pull::Up);

    let pa0 = ExtiInput::new(pa0, p.EXTI0);
    let pa1 = ExtiInput::new(pa1, p.EXTI1);
    let pa2 = ExtiInput::new(pa2, p.EXTI2);

    let pa0_hdl = monitor_pa0(pa0);
    let pa1_hdl = monitor_pa1(pa1);
    let pa2_hdl = monitor_pa2(pa2);

    spawner.spawn(pa0_hdl).unwrap();
    spawner.spawn(pa1_hdl).unwrap();
    spawner.spawn(pa2_hdl).unwrap();

    let (mut rx, mut tx) = buf_usart.split();
    loop {
        let buf = rx.fill_buf().await.unwrap();
        let n = buf.len();
        let byte = rfb_proto::from_bytes(buf);
        rx.consume(n);
        if let Ok(SensorMessage::Request) = byte {
            let count = COUNTER.swap(0, Ordering::Acquire);
            let response = SensorMessage::Response(count as u64);
            let bytes: rfb_proto::Vec<u8, 9> = rfb_proto::to_vec(&response).unwrap();
            tx.write_all(&bytes).await.unwrap();
            tx.flush().await.unwrap();
        }
    }
}

#[embassy_executor::task]
async fn monitor_pa0(mut pa0: ExtiInput<'static, PA0>) {
    loop {
        pa0.wait_for_any_edge().await;
        COUNTER.fetch_add(1, Ordering::Acquire);
        defmt::println!("{}", COUNTER.load(Ordering::SeqCst));
    }
}

#[embassy_executor::task]
async fn monitor_pa1(mut pa1: ExtiInput<'static, PA1>) {
    loop {
        pa1.wait_for_any_edge().await;
        COUNTER.fetch_add(1, Ordering::Acquire);
    }
}

#[embassy_executor::task]
async fn monitor_pa2(mut pa2: ExtiInput<'static, PA2>) {
    loop {
        pa2.wait_for_any_edge().await;
        COUNTER.fetch_add(1, Ordering::Acquire);
    }
}