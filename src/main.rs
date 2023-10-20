#![no_main]
#![no_std]

use embedded_hal::digital::v2::OutputPin;
use nrf52832_hal as hal;
use nrf52832_hal::gpio::Level;
use rtt_target::{rprintln, rtt_init_print};

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();
    let p = hal::pac::Peripherals::take().unwrap();
    let port0 = hal::gpio::p0::Parts::new(p.P0);
    let mut led = port0.p0_17.into_push_pull_output(Level::Low);

    rprintln!("Blinky button demo starting");
    loop {
        led.set_high().unwrap();
        cortex_m::asm::delay(1000000);
        rprintln!("Set LED high");

        led.set_low().unwrap();
        cortex_m::asm::delay(1000000);
        rprintln!("Set LED low");
    }
}
