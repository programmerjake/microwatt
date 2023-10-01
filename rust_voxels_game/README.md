# 3D Voxels Game

# Tools you'll need:

Install Rust using [`rustup`](https://rustup.rs/).

Then run:
```bash
rustup default nightly
rustup target add powerpc64le-unknown-linux-gnu
rustup component add rust-src
cargo install xargo
```

# Run without FPGA/hardware-simulation

Resize your terminal to be at least 100x76.

Building:
```bash
cd rust_voxels_game
cargo build
```

Running:
```bash
cd rust_voxels_game
cargo run
```

# Run on OrangeCrab v0.2.1

Set the OrangeCrab into firmware upload mode by plugging it in to USB while the button is pressed, then run the following commands:

Building/Flashing:
```bash
make -C rust_voxels_game
sudo make FPGA_TARGET=ORANGE-CRAB-0.21 dfuprog DOCKER=1 LITEDRAM_GHDL_ARG=-gUSE_LITEDRAM=false RAM_INIT_FILE=rust_voxels_game/rust_voxels_game.hex MEMORY_SIZE=$((3<<16))
```

Connect a 3.3v USB serial adaptor to the OrangeCrab's TX/RX pins:

pins going from the corner closest to the button:

| Silkscreen Label | Purpose | Connect to on serial adaptor |
|------------------|---------|------------------------------|
| GND              | Ground  | Ground                       |
| 1                | UART RX | TX                           |
| 2                | UART TX | RX                           |

Then, in a separate terminal that you've resized to be at least 100x76, run
(replacing ttyUSB0 with whatever serial device the OrangeCrab is connected to):
```bash
sudo tio --baudrate=1000000 /dev/ttyUSB0
```
