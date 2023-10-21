#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Initializing TWI...");
    let config = twim::Config::default();
    let mut twi = Twim::new(p.TWISPI0, Irqs, p.P0_07, p.P0_08, config);

    let mut expander = sx1509::Sx1509::new(&mut twi, sx1509::DEFAULT_ADDRESS);
    info!("Applying reset");
    unwrap!(expander.borrow(&mut twi).software_reset());

    info!("Setting back direction");
    unwrap!(expander.borrow(&mut twi).set_bank_a_direction(0));

    info!("Reading pins");
    // read the pins from bank a
    let pins = unwrap!(expander.borrow(&mut twi).get_bank_a_data());

    info!("pins: {=u8}", pins);
}
