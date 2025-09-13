# bjr
A Ball Juggling Robot's repository

## Getting started

### firmware build and flashing

install rust on you computer
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Install the target toolchain

```
rustup target add thumbv7em-none-eabihf
```

Install probe-rs
```
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh
```
Download and install jlink tools from here
https://www.segger.com/downloads/jlink/
Restart your shell
Change directory to the main software
```
cd software/central_control_firmware
```

Build the firmware with:
```
cargo embed --bin main_app --release 
```
### telemetry visualizer build
Change directory to the visualizer
```
cd software/telemetry_visualizer
```

Build the visualizer with:
```
cargo embed --bin main_app --release 
```
