#![no_std]
#![no_main]

mod button;
mod channel;
mod executor;
mod future;
mod gpiote;
mod led;
mod time;

use panic_halt as _;

use crate::button::{Button, ButtonDirection};
use crate::channel::Channel;
use crate::executor::run_tasks;
use crate::future::NewFuture;
use crate::led::LedTask;
use crate::time::Ticker;
use cortex_m_rt::entry;
use embedded_hal::digital::OutputPin;
use microbit::board::Board;
use rtt_target::rtt_init_print;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut board = Board::take().unwrap();
    Ticker::init(board.RTC0, &mut board.NVIC);
    let (col, mut row) = board.display_pins.degrade();
    row[0].set_high().unwrap();
    let button_left = board.buttons.button_a.degrade();
    let button_right = board.buttons.button_b.degrade();

    let channel: Channel<ButtonDirection> = Channel::new();
    let mut led_task = LedTask::new(col, channel.get_receiver());
    let mut button_left_task =
        Button::new(button_left, ButtonDirection::Left, channel.get_sender());
    let mut button_right_task =
        Button::new(button_right, ButtonDirection::Right, channel.get_sender());

    let mut tasks: [&mut dyn NewFuture<Output = ()>; 3] =
        [&mut led_task, &mut button_left_task, &mut button_right_task];
    run_tasks(&mut tasks);
}
