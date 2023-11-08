# Flutter BLE Gateway
Foward BLE notification to RabbitMQ queue's.

# How it works?
When starting the application, 
connect to rabbitMQ and start searching for devices via Bluetooth with `deviceName`, 
when it finds connecta on the device, it discovers all services and notification 
features and subscribes to all of them by creating a queue with `"$device_id/$service_id/$characterist_id"`.
Each new notification on the device is forwarded to the rabbitMQ queue.

# I'm not a mobile developer
The ideia is keep as simple as possible, Advertisement search and RabbitMQ user,
password and address is hardcoded and there is only one screen without state.
The scanning run only when app start and if fail you need to restart them, the feedback is just on logs.

# How to run
## Requirements
- [Flutter](https://docs.flutter.dev/get-started/install)

## Run
```bash
flutter run
```
