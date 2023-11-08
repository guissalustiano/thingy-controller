use defmt::*;
use embassy_executor::Spawner;
use nrf_softdevice::ble::peripheral::AdvertiseError;
use nrf_softdevice::ble::{peripheral, Connection};
use nrf_softdevice::{raw, Softdevice};

use core::mem;

use crate::Server;

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

pub fn softdevice_setup<'a, const N: usize>(
    spawner: &'a Spawner,
    device_name: &[u8; N],
) -> (&'a Softdevice, Server) {
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
            p_value: device_name as *const u8 as _,
            current_len: N as u16,
            max_len: N as u16,
            write_perm: unsafe { mem::zeroed() },
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                raw::BLE_GATTS_VLOC_STACK as u8,
            ),
        }),
        ..Default::default()
    };

    let sd = Softdevice::enable(&config);
    let server = unwrap!(Server::new(sd));

    unwrap!(spawner.spawn(softdevice_task(sd)));

    return (sd, server);
}

// https://docs.silabs.com/bluetooth/4.0/general/adv-and-scanning/bluetooth-adv-data-basics
pub async fn advertise_connectable<const N: usize>(
    sd: &Softdevice,
    device_name: &[u8; N],
) -> Result<Connection, AdvertiseError>
where
    [(); N + 9]:,
{
    let adv_data = &mut [0; N + 9];

    adv_data[..9].copy_from_slice(&[
        0x02,
        0x01,
        raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x03,
        0x03,
        0x09,
        0x18,
        (device_name.len() + 1) as u8,
        0x09,
    ]);
    adv_data[9..].copy_from_slice(device_name);

    let config = peripheral::Config::default();
    let scan_data = &[0x03, 0x03, 0x09, 0x18];
    let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
        adv_data,
        scan_data,
    };
    peripheral::advertise_connectable(sd, adv, &config).await
}
