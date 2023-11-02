#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::cell::RefCell;

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Delay;
use embassy_time::Timer;
use static_cell::StaticCell;

use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_sync::blocking_mutex::{NoopMutex, raw::NoopRawMutex};

use mpu9250::Mpu9250;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

static I2C_BUS: StaticCell<NoopMutex<RefCell<Twim<peripherals::TWISPI0>>>> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    info!("Turning on Vdd");
    let mut _vdd_pwd = Output::new(p.P0_30, Level::High, OutputDrive::Standard);

    Timer::after_millis(300).await;

    info!("Initializing TWI...");
    let config = twim::Config::default();
    let i2c = Twim::new(p.TWISPI0, Irqs, p.P0_07, p.P0_08, config);
    let i2c_bus = I2C_BUS.init(NoopMutex::new(RefCell::new(i2c)));

    let mut i2c_dev1 = I2cDevice::new(i2c_bus);
    let mut expander = sx1509::Sx1509::new(&mut i2c_dev1, sx1509::DEFAULT_ADDRESS);
    let mut expander = expander.take(i2c_dev1);
    info!("Applying reset");
    unwrap!(expander.borrow().software_reset());

    info!("Setting back direction");
    unwrap!(expander.borrow().set_bank_a_direction(1));
    unwrap!(expander.borrow().set_bank_b_direction(1));

    info!("Setting pin 1 to output");
    unwrap!(expander.borrow().set_bank_a_data(0x70));
    unwrap!(expander.borrow().set_bank_b_data(0x01)); // Turing on mpu pwd

    let mut i2c_dev2 = I2cDevice::new(i2c_bus);
    let mut mpu = Mpu9250::marg_default(i2c_dev2, &mut Delay).expect("unable to make MPU9250");

    let who_am_i = mpu.who_am_i().expect("could not read WHO_AM_I");
    info!("Who mpu is?: {}", who_am_i);

    loop {
        let data: mpu9250::MargMeasurements<(f32, f32, f32)> = mpu.all().expect("could not read all");
        let acel = data.accel;
        let temp = data.temp;
        let gyro = data.gyro;
        let mag = data.mag;

        info!("acel: {}", acel);
        info!("temp: {}", temp);
        info!("gyro: {}", gyro);
        info!("mag: {}", mag);
        info!("----------------------------------");

        Timer::after_millis(200).await;
    }
}
