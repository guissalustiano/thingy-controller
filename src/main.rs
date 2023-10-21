#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, Pin, Pull, OutputDrive};
use defmt::{info, unwrap};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task(pool_size = 4)]
async fn button_task(
    n: usize,
    mut btn: Input<'static, AnyPin>,
    mut led: Output<'static, AnyPin>,
) {
    loop {
        btn.wait_for_low().await;
        info!("Button {:?} pressed!", n);
        led.set_low();
        info!("Turning on LED {:?}!", n);

        btn.wait_for_high().await;
        info!("Button {:?} released!", n);
        led.set_high();
        info!("Turning off LED {:?}!", n);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Starting!");

    let btn1 = Input::new(p.P0_13.degrade(), Pull::Up);
    let btn2 = Input::new(p.P0_14.degrade(), Pull::Up);
    let btn3 = Input::new(p.P0_15.degrade(), Pull::Up);
    let btn4 = Input::new(p.P0_16.degrade(), Pull::Up);

    let led1 = Output::new(p.P0_17.degrade(), Level::High, OutputDrive::Standard);
    let led2 = Output::new(p.P0_18.degrade(), Level::High, OutputDrive::Standard);
    let led3 = Output::new(p.P0_19.degrade(), Level::High, OutputDrive::Standard);
    let led4 = Output::new(p.P0_20.degrade(), Level::High, OutputDrive::Standard);

    unwrap!(spawner.spawn(button_task(1, btn1, led1)));
    unwrap!(spawner.spawn(button_task(2, btn2, led2)));
    unwrap!(spawner.spawn(button_task(3, btn3, led3)));
    unwrap!(spawner.spawn(button_task(4, btn4, led4)));
}
