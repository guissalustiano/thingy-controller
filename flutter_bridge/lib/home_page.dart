import 'package:flutter/material.dart';
import 'package:flutter_blue_plus/flutter_blue_plus.dart';
import 'package:logger/logger.dart';

var logger = Logger();

class HomePage extends StatefulWidget {
  const HomePage({super.key, required this.title});

  final String title;

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  int _counter = 0;

  void _incrementCounter() {
    setState(() {
      _counter++;
    });
  }

  @override
  initState() {
    super.initState();
    // Add listeners to this class
    FlutterBluePlus.adapterState.listen((BluetoothAdapterState state) {
      logger.d(state);
      if (state != BluetoothAdapterState.on) {
        throw Exception('Bluetooth is off');
      }

      // Setup Listener for scan results.
      FlutterBluePlus.scanResults.listen(
        (results) {
          logger.i('Got results');

          var devices = results
              .where(
                  (e) => e.advertisementData.localName == "Thingy Wii Control")
              .map((e) => e.device)
              .toSet();

          if (devices.isEmpty) {
            logger.i('No devices found');
            return;
          }

          logger.i('Found a devices, stopping scan');
          FlutterBluePlus.stopScan();
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
                        
                        final characteristicSubscription = characteristic.onValueReceived.listen((value) {
                          logger.i('Received value $value from ${characteristic.uuid}');
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
        onError: (e) => logger.e(e.toString()),
        onDone: () => logger.i('scan done'),
      );

      // Start scanning
      FlutterBluePlus.startScan();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: Text(widget.title),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            const Text(
              'You have pushed the button this many times:',
            ),
            Text(
              '$_counter',
              style: Theme.of(context).textTheme.headlineMedium,
            ),
          ],
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _incrementCounter,
        tooltip: 'Increment',
        child: const Icon(Icons.add),
      ),
    );
  }
}
