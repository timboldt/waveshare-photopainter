
```
# Boot to UF2 mode:
cp tools/pico-self-debug.uf2 /media/tboldt/RPI-RP2/`

# Make sure you own the debug port:
sudo chown $USER /dev/hidraw4

# Then do what you would normally do, e.g.:
cargo run

# Or use the debugger in VSCode.
```
