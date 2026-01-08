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
- [Github Awesome Embedded Rust](https://github.com/rust-embedded/awesome-embedded-rust)

### ESP-IDF
- [ESP32-C6 IDF API Reference](https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/api-reference/index.html)
- [Github ESP-IDF-HAL Crate](https://github.com/esp-rs/esp-idf-hal?tab=readme-ov-file)

## Architectural Overview of Embedded Rust via ESP-IDF

```mermaid
---
title: Software Stack
config:
  flowchart:
    defaultRenderer: "elk"
---

flowchart TD
    subgraph "Development Tools - IDE Layer"
        VSCODE[VSCode + rust-analyzer<br/><i>Your chosen IDE</i>]
        PIO[Platform.IO<br/><i>Alternative build system supporting</i><br/><i>both Arduino & ESP-IDF frameworks</i>]
    end
    
    subgraph "Your Application Code"
        APP[Your Rust Application<br/><i>Business logic, sensor loops, WiFi handling</i>]
    end
    
    subgraph "Board Support Layer - BSP"
        BSP_IDF[Board Support Packages C<br/><i>esp-bsp repo: pre-configured drivers for</i><br/><i>dev boards like M5Stack, ESP-BOX, etc.</i>]
        BSP_RUST[Board Support Packages Rust<br/><i>Future: xiao-esp32c6-bsp crates</i><br/><i>Board-specific pin mappings & peripherals</i>]
    end
    
    subgraph "ESP-IDF Path - std Your Choice"
        SVC[esp-idf-svc<br/><i>High-level services: WiFi, MQTT,</i><br/><i>HTTP server, NVS storage, OTA updates</i>]
        HAL_IDF[esp-idf-hal<br/><i>Safe Rust HAL: GPIO, SPI, I2C, UART,</i><br/><i>PWM, ADC, timers - wraps ESP-IDF drivers</i>]
        SYS[esp-idf-sys<br/><i>Raw unsafe FFI bindings via bindgen</i><br/><i>Direct access to all ESP-IDF C functions</i>]
    end
    
    subgraph "Bare Metal Path - no_std Not Your Focus"
        HAL_BARE[esp-hal<br/><i>Pure Rust bare-metal HAL</i><br/><i>Direct register manipulation, no C deps</i>]
        PAC[PAC - Peripheral Access Crate<br/><i>esp32c6, esp32c3, etc. crates</i><br/><i>Auto-generated from SVD files via svd2rust</i><br/><i>Type-safe register access layer</i>]
    end
    
    subgraph "Portable Abstractions - Traits"
        DRIVERS[Community Device Drivers<br/><i>embedded_aht20, dht-embedded, bme280</i><br/><i>Platform-independent sensor libraries</i>]
        EHAL[embedded-hal traits<br/><i>Standard hardware interfaces:</i><br/><i>InputPin, OutputPin, SpiDevice, I2cDevice</i><br/><i>Makes drivers portable across all MCUs</i>]
        ESVC[embedded-svc traits<br/><i>Standard service interfaces:</i><br/><i>WiFi, HTTP, MQTT abstractions</i>]
    end
    
    subgraph "C Framework Layer - Espressif Provided"
        IDF[ESP-IDF Framework<br/><i>Official C framework: peripheral drivers,</i><br/><i>WiFi/BLE stacks, power management, crypto</i>]
        ARDUINO[Arduino Framework<br/><i>Built ON TOP of ESP-IDF as component</i><br/><i>Adds digitalWrite, Serial.print API</i><br/><i>40% larger binaries, limited ESP32-C6 support</i><br/><i>Not used in your Rust setup</i>]
    end
    
    subgraph "Operating System Layer"
        RTOS[FreeRTOS<br/><i>Real-time kernel bundled with ESP-IDF</i><br/><i>Task scheduling, queues, mutexes, semaphores</i><br/><i>Much simpler than Linux - no processes/filesystems</i>]
    end
    
    subgraph "Hardware Layer"
        HW[ESP32-C6 Silicon<br/><i>Memory-mapped registers control:</i><br/><i>GPIO banks, SPI/I2C/UART controllers,</i><br/><i>WiFi 6 radio, BLE 5, ADC, DAC, timers, DMA</i>]
        SENSOR[External Devices<br/><i>XIAO sensors, displays,</i><br/><i>buttons, humidity/temp sensors, motors</i>]
    end
    
    VSCODE -.develops.-> APP
    PIO -.alternative too.-> VSCODE
    PIO -.can build.-> IDF
    PIO -.can build.-> ARDUINO
    
    APP -->|uses services| SVC
    APP -->|reads sensors via| DRIVERS
    APP -->|controls GPIO via| HAL_IDF
    APP -.bare metal route.-> HAL_BARE
    
    BSP_IDF -->|wraps| IDF
    BSP_RUST -->|wraps| HAL_IDF
    APP -.can use.-> BSP_RUST
    
    SVC -->|builds on| HAL_IDF
    SVC -->|direct access when needed| SYS
    SVC -.implements.-> ESVC
    
    DRIVERS -->|require| EHAL
    DRIVERS -->|work with any HAL implementing| HAL_IDF
    DRIVERS -->|work with any HAL implementing| HAL_BARE
    
    HAL_IDF -->|wraps via FFI| SYS
    HAL_IDF -.implements.-> EHAL
    
    HAL_BARE -->|reads/writes| PAC
    HAL_BARE -.implements.-> EHAL
    
    SYS -->|binds to| IDF
    
    PAC -->|generated from SVD| HW
    PAC -->|type-safe register access| HW
    
    IDF -->|uses| RTOS
    IDF -->|configures registers in| HW
    ARDUINO -->|wrapped as component in| IDF
    
    RTOS -->|schedules tasks on| HW
    
    HW <-->|communicates via I2C/SPI/GPIO| SENSOR
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
  - [ESP Wiki](https://www.waveshare.com/wiki/ESP32-P4-Nano-StartPage#Overview)

## First C6 COnnection:

```sh
# sudo dmesg
[  187.639563] usb 1-1: new full-speed USB device number 5 using xhci_hcd
[  187.767728] usb 1-1: New USB device found, idVendor=303a, idProduct=1001, bcdDevice= 1.02
[  187.767745] usb 1-1: New USB device strings: Mfr=1, Product=2, SerialNumber=3
[  187.767751] usb 1-1: Product: USB JTAG/serial debug unit
[  187.767755] usb 1-1: Manufacturer: Espressif
[  187.767759] usb 1-1: SerialNumber: 98:A3:16:8E:C7:C0
[  187.805522] cdc_acm 1-1:1.0: ttyACM0: USB ACM device
[  187.805613] usbcore: registered new interface driver cdc_acm
[  187.805620] cdc_acm: USB Abstract Control Model driver for USB modems and ISDN adapters
```
