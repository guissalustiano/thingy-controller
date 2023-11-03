#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod ble;

use ble::{softdevice_setup, advertise_connectable};
use mpu9250::ImuMeasurements;

use core::cell::RefCell;
use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::{interrupt, bind_interrupts};
use embassy_nrf::gpio::{Input, Pull, Output, Level, OutputDrive};
use embassy_time::{Delay, Timer};
use static_cell::StaticCell;

use embassy_nrf::peripherals::{P0_11, TWISPI0};
use embassy_nrf::twim::{self, Twim};
use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use futures::future::select;
use futures::pin_mut;
use embassy_sync::blocking_mutex::{NoopMutex, raw::NoopRawMutex};
use nrf_softdevice::ble::{gatt_server, Connection};
use sx1509::Sx1509;
use mpu9250::{Mpu9250, Imu, device};


use {defmt_rtt as _, panic_probe as _};

async fn button_service<'a>(btn: &mut Input<'static, P0_11>, server: &'a Server, connection: &'a Connection) {
    loop {
        btn.wait_for_low().await;
        unwrap!(server.buttons.button_notify(connection, &true));

        btn.wait_for_high().await;
        unwrap!(server.buttons.button_notify(connection, &false));
    }
}

async fn mpu_service<'a>(mpu: &mut Mpu9250<device::I2cDevice<I2cDevice<'static, NoopRawMutex, Twim<'static, TWISPI0>>>, Imu>, server: &'a Server, connection: &'a Connection) {
    loop {
        Timer::after_millis(8000).await;
        let data: ImuMeasurements<(f32, f32, f32)> = mpu.all().expect("could not read all");
        let acel = data.accel;
        let gyro = data.gyro;

        unwrap!(server.buttons.accelerometer_x_notify(connection, &0.0));
        unwrap!(server.buttons.accelerometer_y_notify(connection, &1.0));
        unwrap!(server.buttons.accelerometer_z_notify(connection, &2.0));
        info!("accel: {:?}; gyro: {:?};", acel, gyro);
    }
}

#[nrf_softdevice::gatt_service(uuid = "0000DAD0-0000-0000-0000-000000000000")]
pub struct ButtonService {
    #[characteristic(uuid = "0000DAD0-0001-0000-0000-000000000000", notify)]
    button: bool,

    #[characteristic(uuid = "0000DAD0-0002-0000-0000-000000000000", notify)]
    accelerometer_x: f32,

    #[characteristic(uuid = "0000DAD0-0002-0000-0000-000000000001", notify)]
    accelerometer_y: f32,

    #[characteristic(uuid = "0000DAD0-0002-0000-0000-000000000002", notify)]
    accelerometer_z: f32,

    #[characteristic(uuid = "0000DAD0-0003-0000-0000-000000000000", notify)]
    gyroscope_x: f32,

    #[characteristic(uuid = "0000DAD0-0003-0000-0000-000000000001", notify)]
    gyroscope_y: f32,

    #[characteristic(uuid = "0000DAD0-0003-0000-0000-000000000002", notify)]
    gyroscope_z: f32,
}

#[nrf_softdevice::gatt_server]
pub struct Server {
    pub buttons: ButtonService,
}

bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<TWISPI0>;
});

static I2C_BUS: StaticCell<NoopMutex<RefCell<Twim<TWISPI0>>>> = StaticCell::new();


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    const DEVICE_NAME: &'static [u8; 18] = b"Thingy Wii Control";

    // First we get the peripherals access crate.
    let mut config = embassy_nrf::config::Config::default();
    config.gpiote_interrupt_priority = interrupt::Priority::P2;
    config.time_interrupt_priority = interrupt::Priority::P2;
    let p = embassy_nrf::init(config);

    let mut _vdd_pwd = Output::new(p.P0_30, Level::High, OutputDrive::Standard);
    let mut btn = Input::new(p.P0_11, Pull::Up);
    Timer::after_millis(10).await;

    let (sd, server) = softdevice_setup(&spawner, &DEVICE_NAME);

    info!("Initializing TWI...");
    let config = twim::Config::default();
    let i2c = Twim::new(p.TWISPI0, Irqs, p.P0_07, p.P0_08, config);
    let i2c_bus = I2C_BUS.init(NoopMutex::new(RefCell::new(i2c)));

    let mut i2c_dev1 = I2cDevice::new(i2c_bus);
    let expander = Sx1509::new(&mut i2c_dev1, sx1509::DEFAULT_ADDRESS);
    let mut expander = expander.take(i2c_dev1);
    info!("Applying reset");
    unwrap!(expander.borrow().software_reset());

    info!("Setting back direction");
    unwrap!(expander.borrow().set_bank_a_direction(1));
    unwrap!(expander.borrow().set_bank_b_direction(1));

    info!("Setting pin 1 to output");
    unwrap!(expander.borrow().set_bank_a_data(0x70));
    unwrap!(expander.borrow().set_bank_b_data(0x01)); // Turning on mpu pwd

    let i2c_dev2 = I2cDevice::new(i2c_bus);

    let mut mpu = Mpu9250::imu_default(i2c_dev2, &mut Delay).unwrap();

    let who_am_i = mpu.who_am_i().expect("could not read who am i");
    info!("Who mpu is?: {}", who_am_i);

    loop {
        info!("advertising...");
        let conn = unwrap!(advertise_connectable(sd, &DEVICE_NAME).await);
        info!("advertising done! I have a connection.");

        let button_fut = button_service(&mut btn, &server, &conn);
        let mpu_fut = mpu_service(&mut mpu, &server, &conn);

        let gatt_fut = gatt_server::run(&conn, &server, |e| match e {
            ServerEvent::Buttons(e) => match e {
                ButtonServiceEvent::ButtonCccdWrite { notifications } => {
                    info!("button notifications: {}", notifications)
                }
                ButtonServiceEvent::AccelerometerXCccdWrite { notifications } => {
                    info!("accelerometer_x notifications: {}", notifications)
                }
                ButtonServiceEvent::AccelerometerYCccdWrite { notifications } => {
                    info!("accelerometer_y notifications: {}", notifications)
                }
                ButtonServiceEvent::AccelerometerZCccdWrite { notifications } => {
                    info!("accelerometer_z notifications: {}", notifications)
                }
                ButtonServiceEvent::GyroscopeXCccdWrite { notifications } => {
                    info!("gyroscope_x notifications: {}", notifications)
                }
                ButtonServiceEvent::GyroscopeYCccdWrite { notifications } => {
                    info!("gyroscope_y notifications: {}", notifications)
                }
                ButtonServiceEvent::GyroscopeZCccdWrite { notifications } => {
                    info!("gyroscope_z notifications: {}", notifications)
                }
            },
        });

        pin_mut!(gatt_fut);
        pin_mut!(button_fut);
        pin_mut!(mpu_fut);

        select(
            gatt_fut,
            select(
                button_fut,
                mpu_fut,
            )
        ).await;
    }
}
