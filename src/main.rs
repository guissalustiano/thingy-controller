#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::mem;

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::interrupt::InterruptExt;
use embassy_nrf::peripherals::SAADC;
use embassy_nrf::saadc::{AnyInput, Input, Saadc};
use embassy_nrf::{bind_interrupts, interrupt, saadc};
//use embassy_nrf::gpio::{AnyPin, Input, Level, Output, Pin, Pull, OutputDrive};
use embassy_time::{Duration, Timer};

use futures::future::{select, Either};
use futures::pin_mut;
use nrf_softdevice::ble::{gatt_server, peripheral, Connection};
use nrf_softdevice::{raw, Softdevice};

use {defmt_rtt as _, panic_probe as _};

/*
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

//#[embassy_executor::main]
#[allow(dead_code)]
async fn main_old(spawner: Spawner) {
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
*/

bind_interrupts!(struct Irqs {
    SAADC => saadc::InterruptHandler;
});

// https://stackoverflow.com/a/77030892
const fn eval<const N: usize>(data: &[&[u8]]) -> [u8; N] {
    let mut result = [0; N];
    
    let mut i = 0;
    let mut result_i = 0;
    while i < data.len() {
        let mut j = 0;
        while j < data[i].len() {
            result[result_i] = data[i][j];
            result_i += 1;
            j += 1;
        }
        i += 1;
    }
    
    result
}

const fn count_len(arr: &[&[u8]]) -> usize {
    let mut result = 0;
    let mut i = 0;
    while i < arr.len() {
        result += arr[i].len();
        i += 1;
    }
    result
}

macro_rules! declare_const {
    ( const $const_name:ident = [ $($data:tt)* ] ) => {
        const $const_name: [u8; count_len(&[ $($data)* ])] = eval(&[ $($data)* ]);
    };
}

/// Initializes the SAADC peripheral in single-ended mode on the given pin.
fn init_adc(adc_pin: AnyInput, adc: SAADC) -> Saadc<'static, 1> {
    // Then we initialize the ADC. We are only using one channel in this example.
    let config = saadc::Config::default();
    let channel_cfg = saadc::ChannelConfig::single_ended(adc_pin.degrade_saadc());
    interrupt::SAADC.set_priority(interrupt::Priority::P3);
    let saadc = saadc::Saadc::new(adc, Irqs, config, [channel_cfg]);
    saadc
}

/// Reads the current ADC value every second and notifies the connected client.
async fn notify_adc_value<'a>(saadc: &'a mut Saadc<'_, 1>, server: &'a Server, connection: &'a Connection) {
    loop {
        let mut buf = [0i16; 1];
        saadc.sample(&mut buf).await;

        // We only sampled one ADC channel.
        let adc_raw_value: i16 = buf[0];

        // Try and notify the connected client of the new ADC value.
        match server.bas.battery_level_notify(connection, &adc_raw_value) {
            Ok(_) => info!("Battery adc_raw_value: {=i16}", &adc_raw_value),
            Err(_) => unwrap!(server.bas.battery_level_set(&adc_raw_value)),
        };

        // Sleep for one second.
        Timer::after(Duration::from_secs(1)).await
    }
}

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

#[nrf_softdevice::gatt_service(uuid = "180f")]
struct BatteryService {
    #[characteristic(uuid = "2a19", read, notify)]
    battery_level: i16,
}

#[nrf_softdevice::gatt_server]
struct Server {
    bas: BatteryService,
}

const DEVICE_NAME: &'static [u8; 17] = b"Thingy Controller";

fn softdevice_setup(spawner: &Spawner) -> (&Softdevice, Server) {

    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 1,
            event_length: 24,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 256 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: raw::BLE_GATTS_ATTR_TAB_SIZE_DEFAULT.into(),
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: raw::BLE_GAP_ADV_SET_COUNT_DEFAULT as u8,
            periph_role_count: raw::BLE_GAP_ROLE_COUNT_PERIPH_DEFAULT as u8,
            central_role_count: 0,
            central_sec_count: 0,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: DEVICE_NAME as *const u8 as _,
            current_len: DEVICE_NAME.len() as u16,
            max_len: DEVICE_NAME.len() as u16,
            write_perm: unsafe { mem::zeroed() },
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(raw::BLE_GATTS_VLOC_STACK as u8),
        }),
        ..Default::default()
    };

    let sd = Softdevice::enable(&config);
    let server = unwrap!(Server::new(sd));

    unwrap!(spawner.spawn(softdevice_task(sd)));

    return (sd, server);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    // First we get the peripherals access crate.
    let mut config = embassy_nrf::config::Config::default();
    config.gpiote_interrupt_priority = interrupt::Priority::P2;
    config.time_interrupt_priority = interrupt::Priority::P2;
    let p = embassy_nrf::init(config);

    // Then we initialize the ADC. We are only using one channel in this example.
    let adc_pin = p.P0_29.degrade_saadc();
    let mut saadc = init_adc(adc_pin, p.SAADC);
    // Indicated: wait for ADC calibration.
    saadc.calibrate().await;

    let (sd, server) = softdevice_setup(&spawner);

    const BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE: u8  = raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8;
    const PAYLOAD_SIZE: u8 = (DEVICE_NAME.len() + 1) as u8;

    #[rustfmt::skip]
    declare_const!(const ADV_DATA = [
        &[0x02, 0x01, BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE],
        &[0x03, 0x03, 0x09, 0x18],
        &[PAYLOAD_SIZE, 0x09], DEVICE_NAME,
    ]);

    #[rustfmt::skip]
    const SCAN_DATA: [u8; 4] = [0x03, 0x03, 0x09, 0x18];

    #[rustfmt::skip]
    let _adv_data = &[
        0x02, 0x01, raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x03, 0x03, 0x09, 0x18,
        0x0a, 0x09, b'H', b'e', b'l', b'l', b'o', b'R', b'u', b's', b't',
    ];

    loop {
        let config = peripheral::Config::default();

        let adv = peripheral::ConnectableAdvertisement::ScannableUndirected { 
            adv_data: &ADV_DATA, 
            scan_data: &SCAN_DATA,
        };
        let conn = unwrap!(peripheral::advertise_connectable(sd, adv, &config).await);
        info!("advertising done! I have a connection.");

        // We have a GATT connection. Now we will create two futures:
        //  - An infinite loop gathering data from the ADC and notifying the clients.
        //  - A GATT server listening for events from the connected client.
        //
        // Event enums (ServerEvent's) are generated by nrf_softdevice::gatt_server
        // proc macro when applied to the Server struct above
        let adc_fut = notify_adc_value(&mut saadc, &server, &conn);
        let gatt_fut = gatt_server::run(&conn, &server, |e| match e {
            ServerEvent::Bas(e) => match e {
                BatteryServiceEvent::BatteryLevelCccdWrite { notifications } => {
                    info!("battery notifications: {}", notifications)
                }
            },
        });

        pin_mut!(adc_fut);
        pin_mut!(gatt_fut);

        // We are using "select" to wait for either one of the futures to complete.
        // There are some advantages to this approach:
        //  - we only gather data when a client is connected, therefore saving some power.
        //  - when the GATT server finishes operating, our ADC future is also automatically aborted.
        let _ = match select(adc_fut, gatt_fut).await {
            Either::Left((_, _)) => {
                info!("ADC encountered an error and stopped!")
            }
            Either::Right((e, _)) => {
                info!("gatt_server run exited with error: {:?}", e);
            }
        };
    }
}
