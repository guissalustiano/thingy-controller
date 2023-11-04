use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{BasicConsumeArguments, Channel},
    connection::{Connection, OpenConnectionArguments},
    consumer::DefaultConsumer,
};
use tokio::sync::Notify;

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const DEVICE_ID: &str = "DF:89:2B:DA:0B:CB";
const SERVER_UUID: &str = "0000dad0-0000-0000-0000-000000000000";
const BUTTON_UUID: &str = "0000dad0-0001-0000-0000-000000000000";
const ACCELEROMETER_X_UUID: &str = "0000dad0-0002-0000-0000-000000000000";
const ACCELEROMETER_Y_UUID: &str = "0000dad0-0002-0000-0000-000000000001";
const ACCELEROMETER_Z_UUID: &str = "0000dad0-0002-0000-0000-000000000002";
const GYROSCOPE_X_UUID: &str = "0000dad0-0003-0000-0000-000000000000";
const GYROSCOPE_Y_UUID: &str = "0000dad0-0003-0000-0000-000000000001";
const GYROSCOPE_Z_UUID: &str = "0000dad0-0003-0000-0000-000000000002";

async fn default_consumer(channel: &Channel, channel_name: String) {
    log::debug!("inicializing {} consumer", channel_name);
    let args = BasicConsumeArguments::new(&channel_name.to_owned(), &channel_name.to_owned())
        .manual_ack(false)
        .finish();

    channel
        .basic_consume(DefaultConsumer::new(args.no_ack), args)
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    env_logger::init();
    // construct a subscriber that prints formatted traces to stdout
    // global subscriber with log level according to RUST_LOG
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .try_init()
        .ok();

    // open a connection to RabbitMQ server
    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "user",
        "password",
    ))
    .await
    .unwrap();

    /*
    connection
        .register_callback(DefaultConnectionCallback)
        .await
        .unwrap();
    */

    // open a channel on the connection
    let channel = connection.open_channel(None).await.unwrap();
    /*
    channel
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();
    */

    //////////////////////////////////////////////////////////////////////////////
    // start consumer, auto ack
    default_consumer(&channel, format!("{DEVICE_ID}/{SERVER_UUID}/{BUTTON_UUID}")).await;
    default_consumer(&channel, format!("{DEVICE_ID}/{SERVER_UUID}/{ACCELEROMETER_X_UUID}")).await;
    default_consumer(&channel, format!("{DEVICE_ID}/{SERVER_UUID}/{ACCELEROMETER_Y_UUID}")).await;
    default_consumer(&channel, format!("{DEVICE_ID}/{SERVER_UUID}/{ACCELEROMETER_Z_UUID}")).await;
    default_consumer(&channel, format!("{DEVICE_ID}/{SERVER_UUID}/{GYROSCOPE_X_UUID}")).await;
    default_consumer(&channel, format!("{DEVICE_ID}/{SERVER_UUID}/{GYROSCOPE_Y_UUID}")).await;
    default_consumer(&channel, format!("{DEVICE_ID}/{SERVER_UUID}/{GYROSCOPE_Z_UUID}")).await;


    // consume forever
    println!("consume forever..., ctrl+c to exit");
    let guard = Notify::new();
    guard.notified().await;
}
