# Waveshare Photo Painter Project

## Overview

This is an embedded Rust project for the **Waveshare Photo Painter** - a battery-powered RP2040-based device with a 7.3" 7-color e-paper display in a picture frame form factor.

The project uses the **Embassy** async embedded framework for Rust, providing a modern async/await interface for hardware peripherals.

## Hardware Specifications

### Main Components
- **MCU**: Raspberry Pi RP2040 (Cortex-M0+ dual-core)
- **Display**: Waveshare 7.3" EPD (800x480, 7-color e-paper)
  - Colors: Black, White, Red, Orange, Yellow, Green, Blue
  - SPI interface
- **RTC**: PCF85063 Real-Time Clock (I2C interface)
- **Storage**: SD Card slot (currently unused in code)
- **Power**: Battery with USB charging capability
- **Indicators**: 2 LEDs (activity/red, power/green)
- **Input**: User button

### Pin Assignments

#### E-Paper Display (SPI1)
- CLK: PIN_10
- MOSI: PIN_11
- DC (Data/Command): PIN_8
- CS (Chip Select): PIN_9
- RESET: PIN_12
- BUSY: PIN_13
- ENABLE: PIN_16
- DMA: Channel 0

#### RTC (I2C1)
- SDA: PIN_14
- SCL: PIN_15
- INT: PIN_6 (interrupt, not currently used)

#### SD Card (SPI0) - Not Currently Active
- CLK: PIN_2
- MOSI: PIN_3
- MISO: PIN_4
- CS: PIN_5

#### Power & Status
- Activity LED (red): PIN_25
- Power LED (green): PIN_26
- User Button: PIN_19 (active low)
- Battery Enable: PIN_18 (high = enabled)
- Charge State: PIN_17 (low = charging)
- VBUS State: PIN_24 (high = USB power present)
- Power Mode: PIN_23 (purpose unclear)
- VBAT ADC: PIN_29 (battery voltage monitoring)

## Project Structure

```
src/
├── main.rs           # Main application logic and mode selection
├── epaper/
│   ├── mod.rs        # E-paper module exports
│   ├── driver.rs     # EPaper7In3F display driver
│   └── buffer.rs     # DisplayBuffer and pixel manipulation
├── graphics.rs       # Graphics rendering (random walk art)
├── rtc.rs           # PCF85063 RTC driver (I2C)
└── usb_console.rs   # USB serial console interface
```

## Key Dependencies

### Embassy Framework (v0.9/v0.8)
- `embassy-executor` (0.9): Async executor
- `embassy-time` (0.5): Async timers and delays
- `embassy-rp` (0.8): RP2040 HAL with async support
- `embassy-embedded-hal` (0.5): HAL utilities

### Other Dependencies
- `embedded-graphics` (0.8): 2D graphics library
- `embedded-sdmmc` (0.7): SD card support (for future use)
- `defmt`/`defmt-rtt`: Logging framework
- `cortex-m`/`cortex-m-rt`: ARM Cortex-M runtime
- `portable-atomic`: Atomic operations for Cortex-M0+
- `rand`: Random number generation (no_std)

## Current Functionality

### Implemented
1. **E-paper display driver** with full async support
   - 7-color display initialization
   - Image rendering from frame buffer
   - Display clearing (white background)
   - Deep sleep mode
   - Watchdog feeding during long operations

2. **Graphics rendering**
   - Random walk art generation
   - Uses `embedded-graphics` traits
   - Configurable colors and backgrounds

3. **RTC (Real-Time Clock)**
   - Full time reading/writing (year, month, day, hour, minute, second)
   - Time persists across resets (preserved in RTC)
   - Alarm functionality with deep sleep wake-up
   - Proper I2C register access (single-byte writes only)
   - BCD encoding/decoding for time values

4. **USB Console Mode**
   - Automatic detection: enters console mode when USB power detected
   - Serial terminal interface (115200 baud, 8N1)
   - Full command set (see below)
   - Watchdog feeding during console inactivity
   - Command parsing with case-insensitive matching

5. **Power management**
   - Battery voltage monitoring (3.3V * 3 divider)
   - Low battery detection (< 3.1V)
   - USB vs battery power detection
   - Charge state monitoring
   - Automatic power-off on low battery
   - Deep sleep mode with RTC alarm wake-up

6. **Normal Mode** (battery operation)
   - Display update on button press (requires 4 presses)
   - Charging indicator LED
   - Automatic power-off when running on battery
   - Watchdog protection (8s timeout)

### USB Console Commands
When powered via USB, the device enters console mode with the following commands:

- **GO** - Run display update (generates random walk art)
- **CLEAR** - Clear display to white
- **TIME** - Display current RTC time
- **SETTIME Y M D H M S** - Set RTC time (example: `SETTIME 2025 12 6 14 39 30`)
- **SLEEP n** - Deep sleep for n seconds, RTC alarm will wake and power on
- **BATTERY** - Show battery voltage and charging status
- **VERSION** - Show firmware version
- **RESET** - Soft reset the device
- **DFU** - Reboot to USB bootloader (UF2 mode) for firmware updates
- **HELP or ?** - Show command list

### Not Yet Implemented
- [ ] SD card reading for image files
- [ ] Image selection/cycling from SD card
- [ ] Calendar/appointment display from SD card
- [ ] Advanced power optimization
- [ ] Configuration file support

## Build Configuration

### Target
- **Architecture**: `thumbv6m-none-eabi` (Cortex-M0+)
- **Runner**: `elf2uf2-rs -d` (UF2 bootloader)

### Compiler Flags
- Link with custom linker scripts (rp2040, defmt)
- No vectorize loops (optimization)
- Debug symbols enabled in release mode

### Logging
- **DEFMT_LOG**: `trace` level (configured in `.cargo/config.toml`)

## Important Notes

### Memory Layout
The display buffer is **190,800 bytes** (800x480 pixels, 4 bits per pixel) and is allocated as a static mutable:
```rust
static mut DISPLAY_BUF: DisplayBuffer = DisplayBuffer { frame_buffer: [0xFF; 800 * 480 / 2] };
```

### Critical Sections
The RP2040 (Cortex-M0+) lacks native atomic compare-and-swap instructions. The project uses:
- `cortex-m` with `critical-section-single-core` feature
- `portable-atomic` with `critical-section` feature

### Display Timing
- E-paper updates are slow (~seconds for full refresh)
- Watchdog must be fed during display operations
- Display goes to deep sleep between updates

### Battery Considerations
- Minimum voltage: 3.1V
- Voltage divider: 3:1 (measure VSYS which is 3x battery)
- ADC: 12-bit (4096 levels), 3.3V reference
- Calculation: `adc_value * 3300 * 3 / 4096` = millivolts

## Development Workflow

### Building
```bash
cargo build --release
```

### Running (UF2 Bootloader)
```bash
cargo run --release
# Device must be in BOOTSEL mode
# elf2uf2-rs will flash automatically
```

### Alternative: probe-rs
Uncomment in `.cargo/config.toml`:
```toml
runner = "probe-rs run --chip RP2040"
```

### Linting
```bash
cargo clippy --release
```

### Using the USB Console

When the device is powered via USB, it automatically enters console mode:

1. **Connect via USB** - Use a USB-C cable to connect the device
2. **Open a serial terminal** - Use any serial terminal program (picocom, screen, minicom, etc.)
   ```bash
   # macOS/Linux
   picocom -b 115200 /dev/ttyACM0  # or /dev/cu.usbmodem*

   # Or use screen
   screen /dev/ttyACM0 115200
   ```
3. **Enter commands** - Type commands and press Enter
4. **Get help** - Type `HELP` or `?` to see all available commands

Example session:
```
> help
Available commands:
  GO        - Run display update (random art)
  CLEAR     - Clear display to white
  TIME      - Display current RTC time
  SETTIME Y M D H M S - Set RTC time
  ...
> time
Time: 2025-12-06 16:25:03
> battery
Battery: 4200mV
USB power connected, not charging (full)
```

## Architecture Patterns

### Async/Await Throughout
All hardware interactions use async/await:
- SPI transactions
- I2C communication
- Delays and timers
- Display updates

### Error Handling
Custom error types with `Result<T, Error>`:
- E-paper driver: SPI errors, timeouts
- RTC driver: I2C errors, timeouts

### Peripheral Ownership
Peripherals are moved into driver structs with `'static` lifetime:
```rust
pub struct EPaper7In3F<SPI: embassy_rp::spi::Instance + 'static> {
    spi: spi::Spi<'static, SPI, Async>,
    // ... GPIO pins
}
```

## Future Enhancements

### High Priority
1. Implement SD card image loading
2. Add RTC time/date reading
3. Set up RTC alarms for periodic wake-ups
4. Power consumption optimization

### Medium Priority
1. USB logging when powered via USB
2. Multiple image support with cycling
3. Configuration file on SD card
4. Better user interface (button controls)

### Low Priority
1. OTA updates
2. WiFi support (if hardware permits)
3. Advanced graphics/layouts
4. Calendar/appointment display

## Original C Code Reference
The RTC driver was ported from Waveshare's C implementation (v1.0, 2021-02-02). The original behavior is documented at the top of `main.rs`.

## Important Implementation Details

### RTC Register Access
The PCF85063 RTC chip **does not support multi-byte I2C writes** to time and alarm registers. Each register must be written individually in separate I2C transactions. This is critical for reliable operation.

```rust
// CORRECT: Write each register individually
self.i2c.write_async(PCF85063_ADDRESS, [HOURS_REG, value]).await?;
self.i2c.write_async(PCF85063_ADDRESS, [MINUTES_REG, value]).await?;

// WRONG: Multi-byte write will hang/fail
self.i2c.write_async(PCF85063_ADDRESS, [HOURS_REG, hours_val, minutes_val]).await?;
```

### RTC Initialization
The `init()` function intentionally **does not** perform a software reset (unlike the original C code). This preserves the RTC time across device resets. The original C code always set the time immediately after init, so the reset didn't matter. Our implementation preserves time to support persistent timekeeping.

### USB Console Buffer Size
The command parser uses a `heapless::Vec<&str, 8>` for parsing command arguments. This supports commands with up to 8 space-separated parts, which is sufficient for SETTIME (7 parts: command + 6 time values).

## Troubleshooting

### Build Issues
- Ensure Rust nightly or stable 1.75+ is installed
- Check that `thumbv6m-none-eabi` target is installed: `rustup target add thumbv6m-none-eabi`
- Verify `elf2uf2-rs` is installed: `cargo install elf2uf2-rs`

### Runtime Issues
- **Watchdog timeout**: Display operations are properly feeding watchdog, but if you add long operations, ensure `watchdog.feed()` is called regularly. The USB console feeds the watchdog every 4 seconds during inactivity.
- **Display not updating**: Check power enable pin (PIN_16) and verify SPI connections
- **Battery voltage incorrect**: Verify 3:1 voltage divider on VSYS
- **RTC time resets on boot**: Ensure `CONTROL_1_REG` is written with 0x48 (not 0x58) to avoid software reset
- **USB console not appearing**: Verify USB cable supports data (not just charging) and that device detects VBUS (PIN_24 high)

### Common Mistakes to Avoid
- Don't use `Timer::after()` without `await` in async contexts
- Don't forget to feed the watchdog during long operations
- Remember that the display buffer uses 4 bits per pixel (2 pixels per byte)
- X coordinate even/odd determines high/low nibble in buffer
- Don't attempt multi-byte I2C writes to RTC time/alarm registers
- Ensure command parser buffer is large enough for all commands (currently set to 8 parts)

## Contributing Guidelines

When modifying this project:
1. Keep all hardware interactions async
2. Feed the watchdog during operations > 8 seconds
3. Use `defmt::info!()` for logging, not `println!()`
4. Add `#[allow(dead_code)]` for future-use code to keep builds clean
5. Run `cargo clippy` and `cargo +nightly fmt` before committing
6. Update this document when adding new features
