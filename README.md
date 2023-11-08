# Thingy:52 controller for PC
A BLE controller to play games on the computer, is harder but fun.

A side project to learn embedded rust.

# Demo
https://github.com/guissalustiano/thingy-controller/assets/32439070/f664c648-a104-4e70-9645-5d34acdbbbac

# Architecture
![Architecture Overview](docs/imgs/ArchitectureOverview.excalidraw.svg)

## Components
- [Device](thingy-control/)
- [Gateway app](flutter_gateway/)
- [Queue](menssage_broker/)
- [Host Adapter](host-adapter/)
