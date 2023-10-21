#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

// sx1509
const SX1509_ADDRESS: u8 = 0x3e;

bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Initializing TWI...");
    let config = twim::Config::default();
    let mut twi = Twim::new(p.TWISPI0, Irqs, p.P0_07, p.P0_08, config);


    let mask: u8 = 0xFF;

    info!("Applying reset");
    unwrap!(twi.blocking_write(SX1509_ADDRESS, &[0x7D, 0x12]));
    unwrap!(twi.blocking_write(SX1509_ADDRESS, &[0x7D, 0x34]));

    info!("Setting back direction");
    unwrap!(twi.blocking_write(SX1509_ADDRESS, &[0x0F, 0x00]));
    unwrap!(twi.blocking_write(SX1509_ADDRESS, &[0x0E, 0x00]));


    info!("Setting up LED");
    // Write Reg PullUp B
    unwrap!(twi.blocking_write(SX1509_ADDRESS, &[0x07, mask]));
    //
    // Write Reg PullUp A
    unwrap!(twi.blocking_write(SX1509_ADDRESS, &[0x06, mask]));

    info!("Shine!");
}
