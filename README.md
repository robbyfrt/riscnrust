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

### VSCode Extensions
- rust-analyzer
- Wokwi with account and 30day license
- dependi for crates mgmt


## Setup via esp-generate

Use `esp-generate --chip esp32c6 myproject` and follow configuration as necessary with probe-rs

## Run

`cargo run` build the binaries that can then be simulated via Wokwi