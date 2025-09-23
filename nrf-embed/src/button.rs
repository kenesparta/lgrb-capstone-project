use crate::channel::Sender;
use crate::future::{NewFuture, Poll};
use crate::gpiote::InputChannel;
use crate::time::Timer;
use embedded_hal::digital::{InputPin, PinState};
use fugit::ExtU64;
use microbit::hal::gpio::{Floating, Input, Pin};
use microbit::hal::gpiote::Gpiote;

#[derive(Clone, Copy)]
pub enum ButtonDirection {
    Left,
    Right,
}

pub enum ButtonState {
    WaitForPress,
    WaitForRelease,
    Debounce(Timer),
}

pub struct Button<'a> {
    input: InputChannel,
    direction: ButtonDirection,
    state: ButtonState,
    sender: Sender<'a, ButtonDirection>,
}

impl<'a> Button<'a> {
    pub fn new(
        pin: Pin<Input<Floating>>,
        direction: ButtonDirection,
        sender: Sender<'a, ButtonDirection>,
        gpiote: &Gpiote,
    ) -> Self {
        Self {
            input: InputChannel::new(pin, gpiote),
            direction,
            state: ButtonState::WaitForPress,
            sender,
        }
    }
}

impl NewFuture for Button<'_> {
    type Output = ();
    fn poll(&mut self, task_id: usize) -> Poll<Self::Output> {
        loop {
            match self.state {
                ButtonState::WaitForPress => {
                    self.input.set_ready_state(PinState::Low);
                    if let Poll::Ready(()) = self.input.poll(task_id) {
                        self.sender.send(self.direction);
                        self.state = ButtonState::Debounce(Timer::new(100.millis()));
                        continue;
                    }
                }

                ButtonState::Debounce(ref mut timer) => {
                    if let Poll::Ready(()) = timer.poll(task_id) {
                        self.state = ButtonState::WaitForPress;
                        continue;
                    }
                }

                ButtonState::WaitForRelease => {
                    self.input.set_ready_state(PinState::High);
                    if let Poll::Ready(()) = self.input.poll(task_id) {
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
