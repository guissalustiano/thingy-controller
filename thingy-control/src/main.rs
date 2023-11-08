#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod ble;

use ble::{advertise_connectable, softdevice_setup};
use libm::{atan2f, sqrtf};
use mpu9250::ImuMeasurements;

use core::cell::RefCell;
use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::{bind_interrupts, interrupt};
use embassy_time::{Delay, Timer};
use static_cell::StaticCell;

use defmt::Format;
use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_nrf::peripherals::{P0_11, TWISPI0};
use embassy_nrf::twim::{self, Twim};
use embassy_sync::blocking_mutex::{raw::NoopRawMutex, NoopMutex};
use futures::future::select;
use futures::pin_mut;
use mpu9250::{device, Imu, Mpu9250};
use nrf_softdevice::ble::{gatt_server, Connection};
use sx1509::Sx1509;

use {defmt_rtt as _, panic_probe as _};

fn unwrap_notify<T>(result: Result<(), T>, name: &str) {
    match result {
        Ok(_) => info!("{} notify success", name),
        Err(_) => info!("{} notify error", name),
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Format)]
enum LeftRight {
    Left,
    #[default]
    None,
    Right,
}

impl From<LeftRight> for i8 {
    fn from(lr: LeftRight) -> Self {
        match lr {
            LeftRight::Left => 1,
            LeftRight::None => 0,
            LeftRight::Right => -1,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Format)]
enum UpDown {
    Up,
    #[default]
    None,
    Down,
}

impl From<UpDown> for i8 {
    fn from(ud: UpDown) -> Self {
        match ud {
            UpDown::Up => -1,
            UpDown::None => 0,
            UpDown::Down => 1,
        }
    }
}

#[derive(Debug, Default, Format)]
pub struct Control {
    left_right: LeftRight,
    up_down: UpDown,
    shoot: bool,
    jump: bool,
    spin: bool,
}

fn my_incredible_machine_learning_model(
    imu: ImuMeasurements<(f32, f32, f32)>,
    button: bool,
) -> Control {
    let accel = imu.accel;
    let gyro = imu.gyro;

    let pitch = atan2f(accel.0, sqrtf(accel.1 * accel.1 + accel.2 * accel.2));
    let roll = atan2f(accel.1, sqrtf(accel.0 * accel.0 + accel.2 * accel.2));
    Control {
        up_down: match pitch {
            x if x < -0.3 => UpDown::Up,
            x if x > 0.3 => UpDown::Down,
            _ => UpDown::None,
        },
        left_right: match roll {
            x if x > 0.3 => LeftRight::Left,
            x if x < -0.3 => LeftRight::Right,
            _ => LeftRight::None,
        },
        shoot: button,
        jump: accel.2 > -6.5,
        spin: gyro.2 > 3.0,
    }
}

async fn control_service<'a>(
    mpu: &mut Mpu9250<
        device::I2cDevice<I2cDevice<'static, NoopRawMutex, Twim<'static, TWISPI0>>>,
        Imu,
    >,
    btn: &mut Input<'static, P0_11>,
    server: &'a Server,
    connection: &'a Connection,
) {
    let mut previous_control = Control::default();
    loop {
        Timer::after_millis(100).await;
        let data = mpu.all().expect("could not read all");
        let current_control = my_incredible_machine_learning_model(data, btn.is_low());
        notify_control(&previous_control, &current_control, server, connection);
        previous_control = current_control;
    }
}

fn notify_control<'a>(
    previous_state: &Control,
    current_state: &Control,
    server: &'a Server,
    connection: &'a Connection,
) {
    if previous_state.left_right != current_state.left_right {
        info!("left_right: {:?}", current_state.left_right);
        unwrap_notify(
            server
                .control
                .left_right_notify(connection, &current_state.left_right.into()),
            "left_right",
        );
    }

    if previous_state.up_down != current_state.up_down {
        info!("up_down: {:?}", current_state.up_down);
        unwrap_notify(
            server
                .control
                .up_down_notify(connection, &current_state.up_down.into()),
            "up_down",
        );
    }

    if previous_state.shoot != current_state.shoot {
        info!("shoot: {:?}", current_state.shoot);
        unwrap_notify(
            server
                .control
                .shoot_notify(connection, &current_state.shoot),
            "shoot",
        );
    }

    if previous_state.jump != current_state.jump {
        info!("jump: {:?}", current_state.jump);
        unwrap_notify(
            server.control.jump_notify(connection, &current_state.jump),
            "jump",
        );
    }

    if previous_state.spin != current_state.spin {
        info!("spin: {:?}", current_state.spin);
        unwrap_notify(
            server.control.spin_notify(connection, &current_state.spin),
            "spin",
        );
    }
}

#[nrf_softdevice::gatt_service(uuid = "0000DAD0-0000-0000-0000-000000000000")]
pub struct ControlService {
    #[characteristic(uuid = "0000DAD0-0000-0000-0000-000000000001", notify)]
    left_right: i8, // -1 left, 0 none, 1 right

    #[characteristic(uuid = "0000DAD0-0000-0000-0000-000000000002", notify)]
    up_down: i8, // -1 up, 0 none, 1 down

    #[characteristic(uuid = "0000DAD0-0000-0000-0000-000000000003", notify)]
    shoot: bool,

    #[characteristic(uuid = "0000DAD0-0000-0000-0000-000000000004", notify)]
    jump: bool,

    #[characteristic(uuid = "0000DAD0-0000-0000-0000-000000000005", notify)]
    spin: bool,
}

#[nrf_softdevice::gatt_server]
pub struct Server {
    pub control: ControlService,
    //pub sensor: SensorService,
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
    Timer::after_millis(100).await;

    let i2c_dev2 = I2cDevice::new(i2c_bus);

    let mut mpu = Mpu9250::imu_default(i2c_dev2, &mut Delay).unwrap();

    let who_am_i = mpu.who_am_i().expect("could not read who am i");
    info!("Who mpu is?: {}", who_am_i);

    loop {
        info!("advertising...");
        let conn = unwrap!(advertise_connectable(sd, &DEVICE_NAME).await);
        info!("advertising done! I have a connection.");

        let control_fut = control_service(&mut mpu, &mut btn, &server, &conn);

        let gatt_fut = gatt_server::run(&conn, &server, |_e| {
            info!("Connected/Disconnected");
        });

        pin_mut!(gatt_fut);
        pin_mut!(control_fut);

        select(gatt_fut, control_fut).await;
    }
}
