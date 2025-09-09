#![no_std]
#![no_main]

use panic_halt as _; // Panic handler
use cortex_m_rt::entry; // Runtime entry point
use cortex_m::asm::delay;
use stm32f4xx_hal::{
    gpio::{GpioExt, Output, PushPull},
    pac,
    prelude::*,
    rcc::RccExt,
};

#[entry]
fn main() -> ! {
    // Get access to the device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Configure the system clock
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    // Get access to GPIOD (where the onboard LED is typically connected)
    let gpiod = dp.GPIOD.split();

    // Configure PD12 as an output pin (common LED pin on STM32F4 Discovery boards)
    // If your board uses a different pin, change this accordingly
    let mut led = gpiod.pd12.into_push_pull_output();

    // Main loop - blink the LED
    loop {
        // Turn LED on
        led.set_high();

        // Simple delay (not precise, but good enough for blinking)
        delay(1_000_000);

        // Turn LED off
        led.set_low();

        // Another delay
        delay(1_000_000);
    }
}