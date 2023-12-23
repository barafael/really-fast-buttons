#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![forbid(unsafe_code)]

use defmt_rtt as _;
use embassy_futures::select::{select, Either};
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Input, Pull},
    interrupt,
    peripherals::{PA0, PA1, PA2},
    usart::{BufferedUart, Config, State},
};
use embassy_sync::channel::Sender;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embedded_io::asynch::{BufRead, Write};
use rfb_proto::{SensorRequest, SensorResponse};

const ID: &str = env!("CARGO_PKG_NAME");

pub type Message = ();

static CHANNEL: Channel<ThreadModeRawMutex, (), 128> =
    Channel::<ThreadModeRawMutex, (), 128>::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::println!("init: {}", crate::ID);
    let p = embassy_stm32::init(embassy_stm32::Config::default());

    let mut state = State::new();
    let irq = interrupt::take!(USART1);
    let mut tx_buf = [0u8; 32];
    let mut rx_buf = [0u8; 1];
    let mut config = Config::default();
    config.baudrate = 9600;

    let mut buf_usart = BufferedUart::new(
        &mut state,
        p.USART1,
        p.PA10,
        p.PA9,
        irq,
        &mut tx_buf,
        &mut rx_buf,
        config,
    );

    let pa0 = Input::new(p.PA0, Pull::Up);
    let pa1 = Input::new(p.PA1, Pull::Up);
    let pa2 = Input::new(p.PA2, Pull::Up);

    let pa0 = ExtiInput::new(pa0, p.EXTI0);
    let pa1 = ExtiInput::new(pa1, p.EXTI1);
    let pa2 = ExtiInput::new(pa2, p.EXTI2);

    let receiver = CHANNEL.receiver();
    let sender_1 = CHANNEL.sender();
    let sender_2 = CHANNEL.sender();
    let sender_3 = CHANNEL.sender();

    let pa0_hdl = monitor_pa0(pa0, sender_1);
    let pa1_hdl = monitor_pa1(pa1, sender_2);
    let pa2_hdl = monitor_pa2(pa2, sender_3);

    spawner.spawn(pa0_hdl).unwrap();
    spawner.spawn(pa1_hdl).unwrap();
    spawner.spawn(pa2_hdl).unwrap();

    let mut count: u32 = 0;

    let (mut rx, mut tx) = buf_usart.split();
    loop {
        let f = select(receiver.recv(), rx.fill_buf());
        match f.await {
            Either::First(()) => {
                count += 1;
            }
            Either::Second(buf) => {
                defmt::trace!("USART1 active");
                let buf = buf.unwrap();
                let n = buf.len();
                let byte = rfb_proto::from_bytes(buf);
                rx.consume(n);
                if byte == Ok(SensorRequest::GetCount) {
                    let response = SensorResponse::Count(count);
                    count = 0;
                    let bytes: rfb_proto::Vec<u8, 5> = rfb_proto::to_vec(&response).unwrap();
                    tx.write_all(&bytes).await.unwrap();
                    tx.flush().await.unwrap();
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn monitor_pa0(
    mut pa0: ExtiInput<'static, PA0>,
    sender: Sender<'static, ThreadModeRawMutex, (), 128>,
) {
    loop {
        pa0.wait_for_any_edge().await;
        defmt::trace!("PA0 edge");
        sender.send(()).await;
    }
}

#[embassy_executor::task]
async fn monitor_pa1(
    mut pa1: ExtiInput<'static, PA1>,
    sender: Sender<'static, ThreadModeRawMutex, (), 128>,
) {
    loop {
        pa1.wait_for_any_edge().await;
        defmt::trace!("PA1 edge");
        sender.send(()).await;
    }
}

#[embassy_executor::task]
async fn monitor_pa2(
    mut pa2: ExtiInput<'static, PA2>,
    sender: Sender<'static, ThreadModeRawMutex, (), 128>,
) {
    loop {
        pa2.wait_for_any_edge().await;
        defmt::trace!("PA2 edge");
        sender.send(()).await;
    }
}
