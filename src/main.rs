mod wifi;
mod display;
mod timer;
mod blocking_lis3dh;
use blocking_lis3dh::{BlockingI2cAdapter,block_on_lis3dh};

use std::time::Duration;

use embedded_svc::{
    http::{client::Client as HttpClient, Method},
    utils::io};

use esp_idf_hal::{
    gpio::{PinDriver, Pull},
    i2c::{I2cDriver, I2cConfig},
    peripherals::Peripherals,
};
use esp_idf_svc::{
    log::EspLogger,
    nvs::EspDefaultNvsPartition,
    eventloop::EspSystemEventLoop,
    http::client::EspHttpConnection,
};
use ssd1306::{
    I2CDisplayInterface,
    rotation::DisplayRotation::*};
use lis3dh_async::{SlaveAddr,Lis3dh};
use core::cell::RefCell;
use embedded_hal_bus::i2c::RefCellDevice;

use log::{info};

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASSWORD");


fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();
    let mut timer = timer::Timer::new();

    let peripherals = Peripherals::take()?;
    let mut led = PinDriver::output(peripherals.pins.gpio15)?;

    let mut rf_switch = PinDriver::output(peripherals.pins.gpio3)?;
    let mut ext_antenna = PinDriver::output(peripherals.pins.gpio14)?;
    let mut button = PinDriver::input(peripherals.pins.gpio1)?;
    button.set_pull(Pull::Up)?;

    rf_switch.set_low()?; // Set to use the antenna
    ext_antenna.set_high()?; // Set to use the external antenna

    let mut i2c_config = I2cConfig::default();
    i2c_config.baudrate = 100000.into();


    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio22,
        peripherals.pins.gpio23,
        &i2c_config,
    )?;
    
    let shared_bus = RefCell::new(i2c);

    // 2) Wrap each device in a RefCellDevice, all pointing to the same bus
    let imu_i2c  = RefCellDevice::new(&shared_bus);
    let oled_i2c = RefCellDevice::new(&shared_bus);

    // scan_i2c(&mut i2c);
    let interface = I2CDisplayInterface::new(oled_i2c);
    let mut display_mgr = display::DisplayManager::new(interface, Rotate180)?;
    

    if button.is_low() {
        display_mgr.log_and_show("entering imu usage...")?;
        let adapter = BlockingI2cAdapter::new(imu_i2c);
        let mut accelerometer = block_on_lis3dh(
        Lis3dh::new_i2c(adapter, SlaveAddr::Alternate)
        )?;
    
        block_on_lis3dh(accelerometer.set_datarate(lis3dh_async::DataRate::Hz_100))?;
        block_on_lis3dh(accelerometer.set_range(lis3dh_async::Range::G2))?;
        std::thread::sleep(core::time::Duration::from_secs(3));

        loop {
            if button.is_high() { break };
            let a = block_on_lis3dh(accelerometer.accel_raw())?;
            display_mgr.update_line(0,&format!("{:?},{:?},{:?}",a.x,a.y,a.z))?;
            display_mgr.flush()?;
            std::thread::sleep(core::time::Duration::from_millis(100));
        }
    }
    

    led.set_low()?;
    display_mgr.log_and_show("Connecting to WiFi...")?;
    let mut wifi = wifi::WifiManager::new(sys_loop, peripherals.modem, nvs, SSID, PASSWORD)?;
    let ip_info = wifi.get_ip_info()?;
    led.set_high()?;
    display_mgr.log_and_show(&format!("Wifi DHCP info: {ip_info:?}"))?;


    let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);

    // GET
    let result = get_request(&mut client)?;
    display_mgr.log_and_show(&format!("GET: {}", &result))?;

    std::thread::sleep(core::time::Duration::from_secs(3));

    display_mgr.clear()?;
    display_mgr.draw_rect((0,20), (128,64))?;
    let mut some_int: u32 = 0;
    timer.elapsed(); // reset timer
    loop {
        let signal = wifi.get_signal_strength(false)?;
        let btn_state = button.is_low();
        display_mgr.update_line(0, &format!("wifi: {}dB, btn: {}", signal, btn_state))?;
        display_mgr.update_line(1, &format!("cnt: {}, t: {}ms", &some_int, timer.elapsed().as_millis()))?;
        display_mgr.flush().unwrap();

        some_int = some_int.wrapping_add(1);
    }
}

/// Send an HTTP GET request.
fn get_request(client: &mut HttpClient<EspHttpConnection>) -> anyhow::Result<String> {
    // Prepare headers and URL
    let headers = [("accept", "text/plain")];
    let url = "http://ifconfig.net/";

    // Note: If you don't want to pass in any headers, you can also use `client.get(url, headers)`.
    let request = client.request(Method::Get, url, &headers)?;
    info!("-> GET {url}");
    let mut response = request.submit()?;

    // Process response
    let status = response.status();
    info!("<- {status}");

    let mut buf: [u8; _] = [0u8; 0b10_000]; // 10 KB buffer 

    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
    info!("Read {bytes_read} bytes of response body");
    // return response body as &str
    let body = std::str::from_utf8(&buf[0..bytes_read])?.to_owned();
    Ok(body)
}

fn scan_i2c(i2c: &mut I2cDriver) {
    for addr in 0x03u8..0x78u8 {
        let mut buf = [0u8; 1];
        let timeout = Duration::from_millis(10).as_millis() as u32;
        if i2c.read(addr, &mut buf, timeout).is_ok() {
            info!("Found I2C device at 0x{:02X}", addr);
        }
    }
}
