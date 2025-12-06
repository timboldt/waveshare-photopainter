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
├── main.rs           # Main application logic
├── epaper/
│   ├── mod.rs        # E-paper module exports
│   ├── driver.rs     # EPaper7In3F display driver
│   └── buffer.rs     # DisplayBuffer and pixel manipulation
├── graphics.rs       # Graphics rendering (random walk art)
└── rtc.rs           # PCF85063 RTC driver
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
   - Deep sleep mode
   - Watchdog feeding during long operations

2. **Graphics rendering**
   - Random walk art generation
   - Uses `embedded-graphics` traits

3. **RTC initialization**
   - Clock stability check
   - Basic I2C communication

4. **Power management**
   - Battery voltage monitoring (3.3V * 3 divider)
   - Low battery detection (< 3.1V)
   - USB vs battery power detection
   - Charge state monitoring
   - Automatic power-off on low battery

5. **Main loop**
   - Display update on button press (requires 4 presses)
   - Charging indicator LED
   - Runs on USB power, exits on battery
   - Watchdog protection (8s timeout)

### Not Yet Implemented
- [ ] SD card reading for image files
- [ ] RTC time reading/setting
- [ ] RTC alarm functionality
- [ ] USB logging
- [ ] Display refresh rate limiting
- [ ] Image selection/cycling
- [ ] Power optimization

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

## Troubleshooting

### Build Issues
- Ensure Rust nightly or stable 1.75+ is installed
- Check that `thumbv6m-none-eabi` target is installed: `rustup target add thumbv6m-none-eabi`
- Verify `elf2uf2-rs` is installed: `cargo install elf2uf2-rs`

### Runtime Issues
- **Watchdog timeout**: Display operations are properly feeding watchdog, but if you add long operations, ensure `watchdog.feed()` is called regularly
- **Display not updating**: Check power enable pin (PIN_16) and verify SPI connections
- **Battery voltage incorrect**: Verify 3:1 voltage divider on VSYS

### Common Mistakes to Avoid
- Don't use `Timer::after()` without `await` in async contexts
- Don't forget to feed the watchdog during long operations
- Remember that the display buffer uses 4 bits per pixel (2 pixels per byte)
- X coordinate even/odd determines high/low nibble in buffer

## Contributing Guidelines

When modifying this project:
1. Keep all hardware interactions async
2. Feed the watchdog during operations > 8 seconds
3. Use `defmt::info!()` for logging, not `println!()`
4. Add `#[allow(dead_code)]` for future-use code to keep builds clean
5. Run `cargo clippy` and `cargo +nightly fmt` before committing
6. Update this document when adding new features
