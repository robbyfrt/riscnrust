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

## Architectural Overview of Embedded Rust via ESP-IDF

```mermaid
graph TB

    
    subgraph RUST_APP["Your Rust Application Layer"]
        APP_STD["Your std Rust App<br/><i>Uses std library</i>"]
        APP_NOSTD["Your no_std Rust App<br/><i>Bare metal, no allocator</i>"]
    end
    
    subgraph HIGH_SVC["High-Level Service Abstractions"]
        SVC["esp-idf-svc<br/><i>WiFi, MQTT, HTTP server, NVS</i><br/><i>Wraps ESP-IDF services safely</i>"]
    end
    
    subgraph DRIVERS_LAYER["Device Driver Ecosystem"]
        DRIVERS["Community Drivers<br/><i>embedded_aht20, dht-embedded, bme280</i><br/><i>Platform-independent via traits</i>"]
    end
    
    subgraph HAL_LAYER["Hardware Abstraction - Two Paths"]
        direction LR
        subgraph STD_PATH["std Path (ESP-IDF based)"]
            HAL["esp-idf-hal<br/><i>Safe wrappers for GPIO, SPI, I2C, UART</i><br/><i>Builds on ESP-IDF drivers</i>"]
        end
        
        subgraph NOSTD_PATH["no_std Path (Bare Metal)"]
            ESPHAL["esp-hal<br/><i>Pure Rust, direct register access</i><br/><i>No ESP-IDF dependency</i>"]
        end
        
        EHAL["embedded-hal traits<br/><i>Industry standard: InputPin, OutputPin,</i><br/><i>SpiDevice, I2cDevice - portable across MCUs</i>"]
        ESVC["embedded-svc traits<br/><i>Service interfaces: WiFi, HTTP, etc.</i>"]
    end
    
    subgraph BIND_LAYER["FFI Bindings Layer"]
        SYS["esp-idf-sys<br/><i>Unsafe raw bindings via bindgen</i><br/><i>Direct access to all ESP-IDF C functions</i>"]
    end
    
    subgraph C_FRAMEWORK["C Framework Ecosystem"]
        IDF["ESP-IDF Framework<br/><i>Official Espressif SDK in C</i><br/><i>WiFi/BLE stacks, peripheral drivers</i>"]
        ARDUINO["Arduino Framework<br/><i>Simplified C++ API built as ESP-IDF component</i><br/><i>digitalWrite(), Serial, etc.</i>"]
    end
    
    subgraph RTOS_LAYER["Real-Time Operating System"]
        RTOS["FreeRTOS<br/><i>Task scheduler, queues, mutexes, semaphores</i><br/><i>Lightweight RTOS - not a full OS like Linux</i>"]
    end
    
    subgraph HW_LAYER["Silicon & External Hardware"]
        HW["ESP32-C6 / XIAO Hardware<br/><i>RISC-V core, memory-mapped registers</i><br/><i>GPIO, SPI, I2C, ADC, WiFi/BLE radio</i>"]
        SENSOR["External Devices<br/><i>Humidity sensors, displays, buttons, LEDs</i>"]
    end
    
    subgraph TOOLING["Build Toolchain & Flashing"]
        ESPUP["espup<br/><i>Installs Rust toolchain for Xtensa</i><br/><i>RISC-V uses standard Rust</i>"]
        CARGO["cargo / rustc<br/><i>Rust build system</i>"]
        FLASH["cargo-espflash<br/><i>Flashes firmware to device</i>"]
        GDB["GDB<br/><i>Debugger for both paths</i>"]
 				VSCODE["VSCode<br/><i>Your IDE</i>"]
        PIO["PlatformIO<br/><i>Alternative build system/IDE</i><br/><i>Supports C/C++ with Arduino/ESP-IDF</i>"]

    end
    
    
    %% Development environment connections
    VSCODE -->|"compiles with"| CARGO
    VSCODE -->|"flashes with"| FLASH
    VSCODE -->|"debugs with"| GDB
    PIO -.->|"can build Arduino/ESP-IDF"| ARDUINO
    PIO -.->|"can build"| IDF
    
    %% Toolchain connections
    ESPUP -->|"installs for Xtensa chips"| CARGO
    CARGO -->|"builds std projects"| APP_STD
    CARGO -->|"builds no_std projects"| APP_NOSTD
    FLASH -->|"uploads to"| HW
    GDB -->|"debugs on"| HW
    
    %% std application path
    APP_STD -->|"uses high-level services"| SVC
    APP_STD -->|"uses device drivers"| DRIVERS
    APP_STD -->|"uses HAL directly"| HAL
    
    %% no_std application path
    APP_NOSTD -->|"uses device drivers"| DRIVERS
    APP_NOSTD -->|"uses bare metal HAL"| ESPHAL
    
    %% Service layer
    SVC -->|"wraps safely"| HAL
    SVC -->|"calls when needed"| SYS
    SVC -.->|"implements"| ESVC
    
    %% Driver connections
    DRIVERS -->|"programs against traits"| EHAL
    
    %% HAL layers - std path
    HAL -->|"wraps ESP-IDF drivers via"| SYS
    HAL -.->|"implements for portability"| EHAL
    
    %% HAL layers - no_std path
    ESPHAL -->|"writes directly to"| HW
    ESPHAL -.->|"implements for portability"| EHAL
    
    %% Bindings to C framework
    SYS -->|"generates bindings to"| IDF
    
    %% C framework relationships
    ARDUINO -->|"built as component of"| IDF
    IDF -->|"runs on top of"| RTOS
    IDF -->|"configures & drives"| HW
    
    %% RTOS to hardware
    RTOS -->|"manages timing for"| HW
    
    %% Hardware connections
    HW <-->|"communicates via I2C/SPI"| SENSOR
```

## Hardware Overview
- ESP32-C6 from seeedstudio xiao series
  - 160MHz 320KB ROM, 512KB SRAM
  - RiscV core, ble, wifi, thread
  - 20mHz low power core
- Xiao Starter Kit
  - Xiao Expansion Board
    - OLED Display: SSD1306
    - clock: PCF8563
  - several other Grove Sensors
- ePaper Driver board
  - battery & power mgmt: ETA9740
  - 2"-bw-epaper display: ssd1680
- xiao ml kit:
  - https://www.mlsysbook.ai/contents/labs/seeed/xiao_esp32s3/setup/setup.html
  - ESP32-S3 xtensa-based chip
    - dual core 240MHz
    - 8MB PSRAM 8MB Flash, 32GB SD-card 
    - wifi, 14uA deep sleep
  - 6-axis-imu: LSM6DS3TR-C 0x6a
  - camera: OV2640 
  - OLED: SSD1306 0x3c

next:
- esp32-p4
  - dual core 400MHz
  - Memory 
    - 128 KB of high-performance system ROM
    - 768 KB of high-performance (HP) L2 memory (L2MEM)
    - 32 KB of low-power (LP) SRAM
    - 32 MB PSRAM stacked in the package, and the QSPI interface is connected to 16MB Nor Flash
  - additional ESP32-C6 for connectivity
  - PoE, Mipi-CSI, Mipi-DSI
