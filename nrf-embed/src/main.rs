#![no_std]
#![no_main]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    let mut x: usize = 0;
    loop {
        x += 1;
        for _ in 0..10000000 {
            nop()
        }
    }
}
