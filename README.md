# Rust on ESP32-C6

## Prerequisites
- Install rust via `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- restart restart terminal to source rust `. "$HOME/.cargo/env"`
- get ESP Rust Toolchain `cargo install espup`
- setup ESP Rust Toolchain `espup install`
- add `'. /home/robby/export-esp.sh'` to bashrc
- install the template generator `cargo install esp-generate `
- `cargo install espflash --locked`
- `curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh`
- `cargo install esp-config --features=tui --locked`

### For ESP-IDF-HAL Usage
- `sudo apt-get install git wget flex bison gperf python3 python3-pip python3-venv cmake ninja-build ccache libffi-dev libssl-dev dfu-util libusb-1.0-0`
- Do not follow the whole ESP-IDF Installation and instead
- install rust (done)
```sh
cargo install cargo-generate
cargo install ldproxy
# installed previoulsly:
# cargo install espup
# cargo install espflash
# cargo install cargo-espflash # Optional
# sudo apt-get install libudev-dev
```

Finally initialize the folder by executing `cargo generate esp-rs/esp-idf-template cargo`

### VSCode Extensions
- rust-analyzer
- Wokwi with account and 30day license
- dependi for crates mgmt


## Setup via esp-generate

Use `esp-generate --chip esp32c6 myproject` and follow configuration as necessary with probe-rs

## Run

`cargo run` build the binaries that can then be simulated via Wokwi


## Useful Links
- [Three LED on Xiao ESP32-C6](https://wokwi.com/projects/411265368570177537)
- [Wiki - XIAO ESP32-C6 Getting Started ](https://wiki.seeedstudio.com/xiao_esp32c6_getting_started/)
- [Wokwi ESP32 Guide](https://docs.wokwi.com/guides/esp32)

### ESP-IDF
- [ESP32-C6 IDF API Reference](https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/api-reference/index.html)
- [Github ESP-IDF-HAL Crate](https://github.com/esp-rs/esp-idf-hal?tab=readme-ov-file)