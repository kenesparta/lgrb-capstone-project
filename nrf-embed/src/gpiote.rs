use microbit::hal::gpio::{Floating, Input, Pin};
use microbit::hal::gpiote::Gpiote;

pub struct InputChannel {
    pin: Pin<Input<Floating>>,
}

impl InputChannel {
    pub fn new(pin: Pin<Input<Floating>>, gpiote: &Gpiote) -> Self {
        Self { pin }
    }
}