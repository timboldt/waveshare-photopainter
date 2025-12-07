# Waveshare PhotoPainter in Rust

A modern re-implementation in Rust of the firmware for the **Waveshare 7.3" (F) Color E-Paper PhotoPainter** - a battery-powered digital picture frame with a 7-color e-ink display.

## Features

- ✅ **7-Color E-Paper Display** (800x480) - Black, White, Red, Orange, Yellow, Green, Blue
- ✅ **USB Console Interface** - Interactive serial terminal for development and testing
- ✅ **Real-Time Clock** - PCF85063 with persistent timekeeping across resets
- ✅ **Deep Sleep Mode** - RTC alarm-based wake-up for power efficiency
- ✅ **Battery Management** - Voltage monitoring, charging detection, low-battery protection
- ✅ **Random Walk Art** - Built-in procedural art generation
- 🚧 **SD Card Support** - Planned for loading custom images

## Quick Start

### Prerequisites

1. **Install Rust** (1.75 or later):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Add thumbv6m target**:
   ```bash
   rustup target add thumbv6m-none-eabi
   ```

3. **Install elf2uf2-rs**:
   ```bash
   cargo install elf2uf2-rs
   ```

### Building and Flashing

1. **Build the firmware**:
   ```bash
   cargo build --release
   ```

2. **Flash to device** (hold BOOTSEL button while connecting USB):
   ```bash
   cargo run --release
   ```

### Using the USB Console

When powered via USB, the device enters console mode:

1. Connect via USB-C cable
2. Open a serial terminal (115200 baud, 8N1):
   ```bash
   # macOS/Linux
   picocom -b 115200 /dev/ttyACM0

   # Or use screen
   screen /dev/ttyACM0 115200
   ```
3. Type `HELP` to see available commands

#### Available Commands

- `GO` - Generate and display random walk art
- `CLEAR` - Clear display to white
- `TIME` - Show current RTC time
- `SETTIME Y M D H M S` - Set RTC time (e.g., `SETTIME 2025 12 6 14 30 0`)
- `SLEEP n` - Deep sleep for n seconds (RTC alarm wake-up)
- `BATTERY` - Show battery voltage and charging status
- `VERSION` - Show firmware version
- `RESET` - Soft reset device
- `DFU` - Reboot to USB bootloader for firmware updates
- `HELP` or `?` - Show command list

## Hardware

- **MCU**: Raspberry Pi RP2040 (Cortex-M0+ dual-core)
- **Display**: Waveshare 7.3" EPD 7-color e-paper (SPI)
- **RTC**: PCF85063 Real-Time Clock (I2C)
- **Storage**: SD Card slot (future support)
- **Power**: Battery with USB-C charging

## Architecture

Built with modern **async/await** Rust using the **Embassy framework**:
- Non-blocking I/O for all hardware peripherals
- Efficient power management
- Clean separation of concerns
- Type-safe peripheral access

## Documentation

For detailed technical documentation, see [CLAUDE.md](CLAUDE.md).

## License

This project is a re-implementation of Waveshare's original C firmware.

## Acknowledgments

- Original firmware by Waveshare team
- Built with [Embassy](https://embassy.dev/) async embedded framework
