#![no_std]
#![no_main]

mod button;
mod channel;
mod led;
mod time;

use panic_halt as _;

use crate::button::{Button, ButtonDirection};
use crate::channel::Channel;
use crate::led::LedTask;
use crate::time::Ticker;
use cortex_m_rt::entry;
use embedded_hal::digital::OutputPin;
use microbit::board::Board;
use rtt_target::rtt_init_print;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let ticker = Ticker::new(board.RTC0);
    let (col, mut row) = board.display_pins.degrade();
    row[0].set_high().unwrap();
    let button_left = board.buttons.button_a.degrade();
    let button_right = board.buttons.button_b.degrade();

    let channel: Channel<ButtonDirection> = Channel::new();
    let mut led_task = LedTask::new(col, &ticker, channel.get_receiver());
    let mut button_left_task = Button::new(
        button_left,
        &ticker,
        ButtonDirection::Left,
        channel.get_sender(),
    );
    let mut button_right_task = Button::new(
        button_right,
        &ticker,
        ButtonDirection::Right,
        channel.get_sender(),
    );

    loop {
        led_task.poll();
        button_left_task.poll();
        button_right_task.poll();
    }
}
