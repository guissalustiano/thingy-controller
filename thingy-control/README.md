# Thingy:52 BLE Controller
Start a GATT server with the control commands characterists and concurrently 
read the IMU sensor, evaluate the commands and notify the changes.

Everthing made in rust with embassy and nrf-sofdevices binding.

# Setup
## Dependencies:
    - [Rust nightly](https://www.rust-lang.org/tools/install)
    - [probe-rs](https://probe.rs/docs/getting-started/installation/)

## Add Cortex-M4 target
```bash
rustup target add thumbv6m-none-eabi
```

## Compile and flash
```bash
./load_softdevice.sh
cargo run
```

OBS: I need to flash the device (with cargo run) two times before start work
and `--release` didn't work.

# Services and representations
- Controller: `0000DAD0-0000-0000-0000-000000000000`
    - LeftRight: `0000DAD0-0000-0000-0000-000000000001`
        - `-1 = Left`
        -  `0 = None`
        -  `1 = Right`
    - UpDown:    `0000DAD0-0000-0000-0000-000000000002`
        - `-1 = Up`
        -  `0 = None`
        -  `1 = Down`
    - Shoot:    `0000DAD0-0000-0000-0000-000000000003`
        - `0 = False`
        - `1 = True`
    - Jump:     `0000DAD0-0000-0000-0000-000000000004`
        - `0 = False`
        - `1 = True`
    - Spin:    `0000DAD0-0000-0000-0000-000000000005`
        - `0 = False`
        - `1 = True`
