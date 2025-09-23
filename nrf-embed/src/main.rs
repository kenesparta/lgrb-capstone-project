#![no_std]
#![no_main]

mod button;
mod led;

use panic_rtt_target as _;

use button::ButtonDirection;
use embassy_executor::Spawner;
use embassy_nrf::{
    Peri,
    gpio::{AnyPin, Input, Level, Output, OutputDrive, Pull},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Timer;
use futures::{FutureExt, select_biased};
use led::LedRow;
use rtt_target::rtt_init_print;

static CHANNEL: Channel<CriticalSectionRawMutex, ButtonDirection, 1> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    rtt_init_print!();
    let p = embassy_nrf::init(Default::default());

    spawner
        .spawn(button_task(p.P0_14.into(), ButtonDirection::Left))
        .unwrap();
    spawner
        .spawn(button_task(p.P0_23.into(), ButtonDirection::Right))
        .unwrap();

    let _row1 = led_pin(p.P0_21.into());
    let col = [
        led_pin(p.P0_28.into()),
        led_pin(p.P0_11.into()),
        led_pin(p.P0_31.into()),
        led_pin(p.P1_05.into()),
        led_pin(p.P0_30.into()),
    ];

    // LED task:
    let mut blinker = LedRow::new(col);
    loop {
        blinker.toggle();
        select_biased! {
            direction = CHANNEL.receive().fuse() => {
                blinker.shift(direction);
            }
            _ = Timer::after_millis(500).fuse() => {}
        }
    }
}

fn led_pin(pin: Peri<'static, AnyPin>) -> Output<'static> {
    Output::new(pin, Level::High, OutputDrive::Standard)
}

#[embassy_executor::task(pool_size = 2)]
async fn button_task(pin: Peri<'static, AnyPin>, direction: ButtonDirection) {
    let mut input = Input::new(pin, Pull::None);
    loop {
        input.wait_for_low().await;
        CHANNEL.send(direction).await;
        Timer::after_millis(100).await;
        input.wait_for_high().await;
    }
}
