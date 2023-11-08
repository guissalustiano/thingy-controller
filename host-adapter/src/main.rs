use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

use evdev::{uinput::VirtualDeviceBuilder, AttributeSet, InputEvent, KeyCode, KeyEvent};
use lapin::{
    message::DeliveryResult,
    options::{BasicAckOptions, BasicConsumeOptions},
    types::FieldTable,
    Channel, Connection, ConnectionProperties, Consumer,
};
use log::{debug, info};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
enum LeftRight {
    Left,
    #[default]
    None,
    Right,
}

impl From<i8> for LeftRight {
    fn from(lr: i8) -> Self {
        match lr {
            1 => LeftRight::Left,
            0 => LeftRight::None,
            -1 => LeftRight::Right,
            _ => panic!("Invalid value for LeftRight"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
enum UpDown {
    Up,
    #[default]
    None,
    Down,
}

impl From<i8> for UpDown {
    fn from(ud: i8) -> Self {
        match ud {
            -1 => UpDown::Up,
            0 => UpDown::None,
            1 => UpDown::Down,
            _ => panic!("Invalid value for UpDown"),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Control {
    left_right: LeftRight,
    up_down: UpDown,
    shoot: bool,
    jump: bool,
    spin: bool,
}

const DEVICE_ID: &str = "DF:89:2B:DA:0B:CB";
const CONTROL_SERVER_UUID: &str = "0000dad0-0000-0000-0000-000000000000";
const LEFT_RIGHT_UUID: &str = "0000dad0-0000-0000-0000-000000000001";
const UP_DOWN_UUID: &str = "0000dad0-0000-0000-0000-000000000002";
const SHOOT_UUID: &str = "0000dad0-0000-0000-0000-000000000003";
const JUMP_UUID: &str = "0000dad0-0000-0000-0000-000000000004";
const SPIN_UUID: &str = "0000dad0-0000-0000-0000-000000000005";

static CONTROL_STATE: Lazy<Arc<Mutex<Control>>> =
    Lazy::new(|| Arc::new(Mutex::new(Control::default())));

async fn create_consumer(
    channel: &Channel,
    characterist_uuid: &str,
) -> Result<Consumer, lapin::Error> {
    let queue_name = format!("{DEVICE_ID}/{CONTROL_SERVER_UUID}/{characterist_uuid}");
    channel
        .basic_consume(
            queue_name.as_str(),
            queue_name.as_str(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let uri = "amqp://user:password@localhost:5672/%2f";
    let options = ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);

    let connection = Connection::connect(uri, options).await.unwrap();
    let channel = connection.create_channel().await.unwrap();

    // Read queue and update CONTROL_STATE
    create_consumer(&channel, LEFT_RIGHT_UUID)
        .await
        .map(|consumer| {
            consumer.set_delegate(move |delivery: DeliveryResult| async {
                let delivery = match delivery {
                    Err(_) | Ok(None) => return,
                    Ok(Some(delivery)) => delivery,
                };

                {
                    let value: i8 = delivery.data[0] as i8;
                    let mut control = CONTROL_STATE.lock().unwrap();
                    control.left_right = value.into();
                    debug!("RECEIVE left_right: {:?}", control.left_right);
                }

                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("Failed to ack send_webhook_event message");
            });
        })?;

    create_consumer(&channel, UP_DOWN_UUID)
        .await
        .map(|consumer| {
            consumer.set_delegate(move |delivery: DeliveryResult| async {
                let delivery = match delivery {
                    Err(_) | Ok(None) => return,
                    Ok(Some(delivery)) => delivery,
                };

                {
                    let value: i8 = delivery.data[0] as i8;
                    let mut control = CONTROL_STATE.lock().unwrap();
                    control.up_down = value.into();
                    debug!("RECEIVE up_down: {:?}", control.up_down);
                }

                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("Failed to ack send_webhook_event message");
            });
        })?;

    create_consumer(&channel, SHOOT_UUID)
        .await
        .map(|consumer| {
            consumer.set_delegate(move |delivery: DeliveryResult| async {
                let delivery = match delivery {
                    Err(_) | Ok(None) => return,
                    Ok(Some(delivery)) => delivery,
                };

                {
                    let value = delivery.data[0] != 0;
                    let mut control = CONTROL_STATE.lock().unwrap();
                    control.shoot = value;
                    debug!("RECEIVE shoot: {:?}", control.shoot);
                }

                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("Failed to ack send_webhook_event message");
            });
        })?;

    create_consumer(&channel, JUMP_UUID).await.map(|consumer| {
        consumer.set_delegate(move |delivery: DeliveryResult| async {
            let delivery = match delivery {
                Err(_) | Ok(None) => return,
                Ok(Some(delivery)) => delivery,
            };

            {
                let value = delivery.data[0] != 0;
                let mut control = CONTROL_STATE.lock().unwrap();
                control.jump = value;
                debug!("RECEIVE jump: {:?}", control.jump);
            }

            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("Failed to ack send_webhook_event message");
        });
    })?;

    create_consumer(&channel, SPIN_UUID).await.map(|consumer| {
        consumer.set_delegate(move |delivery: DeliveryResult| async {
            let delivery = match delivery {
                Err(_) | Ok(None) => return,
                Ok(Some(delivery)) => delivery,
            };

            {
                let value = delivery.data[0] != 0;
                let mut control = CONTROL_STATE.lock().unwrap();
                control.spin = value;
                debug!("RECEIVE spin: {:?}", control.spin);
            }

            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("Failed to ack send_webhook_event message");
        });
    })?;

    // Dispach Keyboard events
    tokio::spawn(async move {
        let mut previous_control = Control::default();
        let mut keys_set = AttributeSet::new();
        for key in [
            KeyCode::KEY_UP,
            KeyCode::KEY_DOWN,
            KeyCode::KEY_LEFT,
            KeyCode::KEY_RIGHT,
            KeyCode::KEY_X,
            KeyCode::KEY_Z,
            KeyCode::KEY_C,
        ] {
            keys_set.insert(key);
        }

        let mut device = VirtualDeviceBuilder::new()
            .unwrap()
            .name("ESP Wii Controller")
            .with_keys(&keys_set)
            .unwrap()
            .build()
            .unwrap();

        loop {
            let current_control = (*CONTROL_STATE.lock().unwrap()).clone();
            let mut keys_events = Vec::new();

            if previous_control.left_right != current_control.left_right {
                info!(
                    "left_right: {:?} to {:?}",
                    previous_control.left_right, current_control.left_right
                );
                match previous_control.left_right {
                    LeftRight::Left => keys_events.push(KeyCode::KEY_LEFT.release()),
                    LeftRight::Right => keys_events.push(KeyCode::KEY_RIGHT.release()),
                    LeftRight::None => {}
                }

                match current_control.left_right {
                    LeftRight::Left => keys_events.push(KeyCode::KEY_LEFT.press()),
                    LeftRight::Right => keys_events.push(KeyCode::KEY_RIGHT.press()),
                    LeftRight::None => {}
                }
            }

            if previous_control.up_down != current_control.up_down {
                info!(
                    "up_down: {:?} to {:?}",
                    previous_control.up_down, current_control.up_down
                );
                match previous_control.up_down {
                    UpDown::Up => keys_events.push(KeyCode::KEY_UP.release()),
                    UpDown::Down => keys_events.push(KeyCode::KEY_DOWN.release()),
                    UpDown::None => {}
                }

                match current_control.up_down {
                    UpDown::Up => keys_events.push(KeyCode::KEY_UP.press()),
                    UpDown::Down => keys_events.push(KeyCode::KEY_DOWN.press()),
                    UpDown::None => {}
                }
            }

            if previous_control.shoot != current_control.shoot {
                info!(
                    "shoot: {:?} to {:?}",
                    previous_control.shoot, current_control.shoot
                );
                if current_control.shoot {
                    keys_events.push(KeyCode::KEY_X.press());
                } else {
                    keys_events.push(KeyCode::KEY_X.release());
                }
            }

            if previous_control.jump != current_control.jump {
                info!(
                    "jump: {:?} to {:?}",
                    previous_control.jump, current_control.jump
                );
                if current_control.jump {
                    keys_events.push(KeyCode::KEY_Z.press());
                } else {
                    keys_events.push(KeyCode::KEY_Z.release());
                }
            }

            if previous_control.spin != current_control.spin {
                info!(
                    "spin: {:?} to {:?}",
                    previous_control.spin, current_control.spin
                );
                if current_control.spin {
                    keys_events.push(KeyCode::KEY_C.press());
                } else {
                    keys_events.push(KeyCode::KEY_C.release());
                }
            }

            previous_control = current_control;
            device.emit(&keys_events[..]).unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    });

    std::future::pending::<()>().await;

    Ok(())
}

trait KeyCodeExtension {
    fn release(&self) -> InputEvent;
    fn press(&self) -> InputEvent;
}
impl KeyCodeExtension for KeyCode {
    fn release(&self) -> InputEvent {
        *KeyEvent::new(KeyCode(self.code()), 0)
    }
    fn press(&self) -> InputEvent {
        *KeyEvent::new(KeyCode(self.code()), 1)
    }
}
