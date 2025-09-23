use crate::executor::wake_task;
use crate::future::{NewFuture, Poll};
use core::cell::{RefCell, RefMut};
use core::sync::atomic::{AtomicU32, Ordering};
use critical_section::Mutex;
use fugit::{Duration, Instant};
use heapless::BinaryHeap;
use heapless::binary_heap::Min;
use microbit::hal::rtc::{Rtc, RtcInterrupt};
use microbit::pac::{NVIC, RTC0, interrupt};
use nrf52833_hal::rtc::RtcCompareReg;
use nrf52833_hal::rtc::RtcInterrupt::Tick;

type TickInstant = Instant<u64, 1, 32768>;
type TickDuration = Duration<u64, 1, 32768>;

const MAX_DEADLINES: usize = 4;
static WAKE_DEADLINES: Mutex<RefCell<BinaryHeap<(u64, usize), Min, MAX_DEADLINES>>> =
    Mutex::new(RefCell::new(BinaryHeap::new()));

fn schedule_wakeup(
    mut rm_deadlines: RefMut<BinaryHeap<(u64, usize), Min, MAX_DEADLINES>>,
    mut rm_rtc: RefMut<Option<Rtc<RTC0>>>,
) {
    let rtc = rm_rtc.as_mut().unwrap();
    while let Some((deadline, task_id)) = rm_deadlines.peek() {
        let ovf_count = (*deadline >> 24) as u32;
        if ovf_count == TICKER.ovf_count.load(Ordering::Relaxed) {
            let counter = (*deadline & 0xFF_FF_FF) as u32;
            if counter > (rtc.get_counter() + 1) {
                rtc.set_compare(RtcCompareReg::Compare0, counter).ok();
                rtc.enable_event(RtcInterrupt::Compare0);
            } else {
                wake_task(*task_id);
                rm_deadlines.pop();
                continue;
            }
        }
        break;
    }

    if rm_deadlines.is_empty() {
        rtc.disable_event(RtcInterrupt::Compare0);
    }
}

enum TimerState {
    Init,
    Wait,
}

pub struct Timer {
    end_time: TickInstant,
    state: TimerState,
}

impl Timer {
    pub fn new(duration: TickDuration) -> Self {
        Self {
            end_time: Ticker::now() + duration,
            state: TimerState::Init,
        }
    }
    fn register(&self, task_id: usize) {
        let new_deadline = self.end_time.ticks();
        critical_section::with(|cs| {
            let mut rm_deadlines = WAKE_DEADLINES.borrow_ref_mut(cs);
            let is_earliest = if let Some((next_deadline, _)) = rm_deadlines.peek() {
                new_deadline < *next_deadline
            } else {
                true
            };

            if rm_deadlines.push((new_deadline, task_id)).is_err() {
                panic!("Deadline dropped for task {}", task_id);
            }

            if is_earliest {
                schedule_wakeup(rm_deadlines, TICKER.rtc.borrow_ref_mut(cs));
            }
        })
    }
}

impl NewFuture for Timer {
    type Output = ();

    fn poll(&mut self, task_id: usize) -> Poll<Self::Output> {
        match self.state {
            TimerState::Init => {
                self.register(task_id);
                self.state = TimerState::Wait;
                Poll::Pending
            }

            TimerState::Wait => {
                if Ticker::now() >= self.end_time {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

static TICKER: Ticker = Ticker {
    ovf_count: AtomicU32::new(0),
    rtc: Mutex::new(RefCell::new(None)),
};

pub struct Ticker {
    ovf_count: AtomicU32,
    rtc: Mutex<RefCell<Option<Rtc<RTC0>>>>,
}

impl Ticker {
    pub fn init(rtc0: RTC0, nvic: &mut NVIC) {
        let mut rtc = Rtc::new(rtc0, 0).unwrap();
        rtc.enable_counter();

        #[cfg(feature = "trigger-overflow")]
        {
            rtc.trigger_overflow();
            while rtc.get_counter() == 0 {}
        }

        rtc.enable_event(RtcInterrupt::Overflow);
        rtc.enable_interrupt(RtcInterrupt::Overflow, Some(nvic));
        rtc.enable_interrupt(RtcInterrupt::Compare0, Some(nvic));
        critical_section::with(|cs| TICKER.rtc.replace(cs, Some(rtc)));
    }

    pub fn now() -> TickInstant {
        let ticks = {
            loop {
                let ovf_before = TICKER.ovf_count.load(Ordering::SeqCst);
                let counter = critical_section::with(|cs| {
                    TICKER.rtc.borrow_ref(cs).as_ref().unwrap().get_counter()
                });
                let ovf = TICKER.ovf_count.load(Ordering::SeqCst);
                if ovf_before == ovf {
                    break (ovf as u64) << 24 | counter as u64;
                }
            }
        };
        TickInstant::from_ticks(ticks)
    }
}

#[interrupt]
fn RTC0() {
    critical_section::with(|cs| {
        let mut rm_rtc = TICKER.rtc.borrow_ref_mut(cs);
        let rtc = rm_rtc.as_mut().unwrap();

        if rtc.is_event_triggered(RtcInterrupt::Overflow) {
            rtc.reset_event(RtcInterrupt::Overflow);
            TICKER.ovf_count.fetch_add(1, Ordering::Relaxed);
        }

        if rtc.is_event_triggered(RtcInterrupt::Compare0) {
            rtc.reset_event(RtcInterrupt::Compare0);
        }

        schedule_wakeup(WAKE_DEADLINES.borrow_ref_mut(cs), rm_rtc);
    })
}
