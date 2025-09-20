use crate::channel::Sender;
use crate::future::{NewFuture, Poll};
use crate::time::Timer;
use embedded_hal::digital::InputPin;
use fugit::ExtU64;
use microbit::hal::gpio::{Floating, Input, Pin};

#[derive(Clone, Copy)]
pub enum ButtonDirection {
    Left,
    Right,
}

pub enum ButtonState {
    WaitForPress,
    Debounce(Timer),
}

pub struct Button<'a> {
    pin: Pin<Input<Floating>>,
    direction: ButtonDirection,
    state: ButtonState,
    sender: Sender<'a, ButtonDirection>,
}

impl<'a> Button<'a> {
    pub fn new(
        pin: Pin<Input<Floating>>,
        direction: ButtonDirection,
        sender: Sender<'a, ButtonDirection>,
    ) -> Self {
        Self {
            pin,
            direction,
            state: ButtonState::WaitForPress,
            sender,
        }
    }
}

impl NewFuture for Button<'_> {
    type Output = ();
    fn poll(&mut self, task_id: usize) -> crate::future::Poll<()> {
        loop {
            match self.state {
                ButtonState::WaitForPress => {
                    if self.pin.is_low().unwrap() {
                        self.sender.send(self.direction);
                        self.state = ButtonState::Debounce(Timer::new(100.millis()));
                        continue;
                    }
                }
                ButtonState::Debounce(ref timer) => {
                    if timer.is_ready() && self.pin.is_high().unwrap() {
                        self.state = ButtonState::WaitForPress;
                        continue;
                    }
                }
            }
            break;
        }

        Poll::Pending
    }
}
