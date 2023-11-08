# Thingy:52 controller for PC
A BLE controller to play games on the computer, it is harder but fun.

A side project to learn embbeed rust.

# Architecture
![Architecture Overview](docs/imgs/ArchitectureOverview.excalidraw.svg)

# Device BLE services and representations
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
