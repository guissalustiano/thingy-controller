import 'package:flutter/material.dart';
import 'package:flutter_blue_plus/flutter_blue_plus.dart';
import "package:dart_amqp/dart_amqp.dart";
import 'package:logger/logger.dart';
import 'dart:typed_data';

var logger = Logger();

class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  static const rabbitMqHost = "145.126.47.164";
  static const deviceName = "Thingy Wii Control";

  bool _scanning = true;

  @override
  initState() {
    super.initState();
    // Add listeners to this class
    FlutterBluePlus.adapterState.listen((BluetoothAdapterState state) {
      logger.d(state);
      if (state != BluetoothAdapterState.on) {
        throw Exception('Bluetooth is off');
      }

      ConnectionSettings settings = ConnectionSettings(
        host: rabbitMqHost,
        authProvider: const PlainAuthenticator("user", "password")
      );
      Client client = Client(settings: settings);

      client.channel().then((channel) async {
        // Setup Listener for scan results.
        FlutterBluePlus.scanResults.listen(
          (results) {
            logger.i('Got results');

            var devices = results
                .where(
                    (e) => e.advertisementData.localName == deviceName)
                .map((e) => e.device)
                .toSet();

            if (devices.isEmpty) {
              logger.i('No devices found');
              return;
            }

            logger.i('Found a devices, stopping scan');
            FlutterBluePlus.stopScan();
            setState(() {
              _scanning = false;
            });
            
            for (var device in devices) {
              logger.i('Connecting to ${device.platformName}');

              device.connectionState
                  .listen((BluetoothConnectionState state) async {
                if (state != BluetoothConnectionState.connected) {
                  logger.w('Not connected to ${device.platformName}');
                  return;
                }
                logger.i('Connected to ${device.platformName}');
                List<BluetoothService> services = await device.discoverServices();
                for (var service in services) {
                logger.i('Looking service ${service.uuid}');
                  var characteristics = service.characteristics;
                  for(var characteristic in characteristics) {
                      logger.i('Looking characteristic ${characteristic.uuid}');
                      if (characteristic.properties.notify) {
                          logger.i('Found notify characteristic ${characteristic.uuid}');
                          var queue = await channel.queue(
                            "${device.remoteId}/${service.uuid}/${characteristic.uuid}",
                            arguments: {
                              "x-message-ttl": 1000,
                            }
                          );
                          
                          final characteristicSubscription = characteristic.onValueReceived.listen((value) {
                            var byteArray = Uint8List.fromList(value);
                            logger.i('Received value $byteArray from ${characteristic.uuid}');
                            queue.publish(byteArray);
                        });
                        device.cancelWhenDisconnected(characteristicSubscription);
                        await characteristic.setNotifyValue(true);
                      }
                  }
                }
              });

              device.connect();
            }
          },
          onError: (e) {
            logger.e(e.toString());
            client.close();
          },
          onDone: (){
            logger.i('scan done');
            client.close();
          },
        );
      });

      // Start scanning
      FlutterBluePlus.startScan();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: const Text("BLE-Queue Bridge"),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            (_scanning ?
                const Text("Scanning for devices...") :
                const Text("Forwarding $deviceName to $rabbitMqHost")
            ),
          ],
        ),
      )
    );
  }
}
