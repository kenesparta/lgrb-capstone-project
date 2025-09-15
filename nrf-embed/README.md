# Capstone Project: Embedded part

# 🚀 Getting started
1. Install the tools, please take a look at this [link 🔗](https://docs.rust-embedded.org/book/intro/tooling.html)
2. Install dgb, openocd and qemu, please take a look at these resources:
   - [Linux](https://docs.rust-embedded.org/book/intro/install/linux.html)
   - [MacOS](https://docs.rust-embedded.org/book/intro/install/macos.html)
   - [Windows](https://docs.rust-embedded.org/book/intro/install/windows.html)
3. Install [probe-rs](https://probe.rs/)
4. Run `make prepare` to install the required dependencies

# ⚡️ Running the project
- We are using [this arm platform](https://doc.rust-lang.org/nightly/rustc/platform-support/thumbv7em-none-eabi.html).

# The cortex-m-rt crate
- Here, we have memory.x

![memory-map](memory-map.png)

# 📚 Resources
- [pcb-reflow-stm32-rust-rtic](https://github.com/marcinwionczyk/pcb-reflow-stm32-rust-rtic)
- [The rusty Bits - Embedded Rust setup explained](https://www.youtube.com/watch?v=TOAynddiu5M)
- [The rusty Bits - Blinking an LED: Embedded Rust ecosystem explored](https://www.youtube.com/watch?v=A9wvA_S6m7Y)

## Datasheet
- [nRF52 Application Processor](https://tech.microbit.org/hardware/#nrf52-application-processor)
- [Product Specification nRF52840](https://docs-be.nordicsemi.com/bundle/ps_nrf52833/attach/nRF52833_PS_v1.7.pdf?_LANG=enus)
