use crate::executor::wake_task;
use crate::future::{NewFuture, Poll};
use core::sync::atomic::Ordering::Relaxed;
use core::sync::atomic::{AtomicUsize, Ordering::Acquire};
use cortex_m::peripheral::NVIC;
use embedded_hal::digital::{InputPin, PinState};
use microbit::hal::gpio::{Floating, Input, Pin};
use microbit::hal::gpiote::Gpiote;
use microbit::hal::pac::{Interrupt, interrupt};

const MAX_CHANNELS_USED: usize = 2;
static NEXT_CHANNEL: AtomicUsize = AtomicUsize::new(0);

pub struct InputChannel {
    pin: Pin<Input<Floating>>,
    channel_id: usize,
    ready_state: PinState,
}

impl InputChannel {
    pub fn new(pin: Pin<Input<Floating>>, gpiote: &Gpiote) -> Self {
        let channel_id = NEXT_CHANNEL.fetch_add(1, Acquire);
        let channel = match channel_id {
            0 => gpiote.channel0(),
            1 => gpiote.channel1(),
            MAX_CHANNELS_USED.. => panic!("Too many input channels"),
        };
        channel.input_pin(&pin).toggle().enable_interrupt();
        unsafe { NVIC::unmask(Interrupt::GPIOTE) }
        Self {
            pin,
            channel_id,
            ready_state: PinState::Low,
        }
    }

    pub fn set_ready_state(&mut self, ready_state: PinState) {
        self.ready_state = ready_state;
    }
}

const INVALID_TASK_ID: usize = 0xFFFF_FFFF;
const DEFAULT_TASK: AtomicUsize = AtomicUsize::new(INVALID_TASK_ID);
static WAKE_TASKS: [AtomicUsize; MAX_CHANNELS_USED] = [DEFAULT_TASK; MAX_CHANNELS_USED];

impl NewFuture for InputChannel {
    type Output = ();
    fn poll(&mut self, task_id: usize) -> Poll<Self::Output> {
        if self.ready_state == PinState::from(self.pin.is_high().unwrap()) {
            return Poll::Ready(());
        }
        WAKE_TASKS[self.channel_id].store(task_id, Relaxed);
        Poll::Pending
    }
}

#[interrupt]
fn GPIOTE() {
    unsafe {
        let gpiote = &*microbit::pac::GPIOTE::ptr();
        for (channel, task) in WAKE_TASKS.iter().enumerate() {
            if gpiote.events_in[channel].read().bits() != 0 {
                gpiote.events_in[channel].write(|w| w);
                let task_id = task.swap(INVALID_TASK_ID, Acquire);
                if task_id != INVALID_TASK_ID {
                    wake_task(task_id);
                }
            }
        }
        gpiote.events_in[0].read().bits();
    }
}
