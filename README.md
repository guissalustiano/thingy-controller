# Thingy:52 controller for PC
A BLE controller to play games on the computer, is harder but fun.

A side project to learn embedded rust.

# Demo
https://github.com/guissalustiano/thingy-controller/assets/32439070/f664c648-a104-4e70-9645-5d34acdbbbac

# Architecture
![Architecture Overview](docs/imgs/ArchitectureOverview.excalidraw.svg)

[Presentation slides](https://docs.google.com/presentation/d/e/2PACX-1vTnos5OwoKDWusqpnd-sa6tKcuvkKpwxMkUwGILVSXXwLN9TXxYZPakCfE7Ar3PiUT9J4IHL9ZTX4Jt/pub?start=false&loop=false&delayms=3000)

## Components
- [Device](thingy-control/) 308 LoC
- [Gateway app](flutter_gateway/) - 136 LoC
- [Queue](menssage_broker/) - 0 LoC
- [Host Adapter](host-adapter/) - 289 LoC
