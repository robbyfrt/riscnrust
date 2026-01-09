use embedded_svc::{
    http::{client::Client as HttpClient, Method},
    utils::io};

use esp_idf_hal::{
    gpio::PinDriver,
    i2c::{I2cDriver, I2cConfig},
    peripherals::Peripherals,
    delay::FreeRtos
};
use esp_idf_svc::{
    log::EspLogger,
    wifi::{
        BlockingWifi,
        EspWifi,
        Configuration,
        ClientConfiguration,
        AuthMethod,
    },
    nvs::EspDefaultNvsPartition,
    eventloop::EspSystemEventLoop,
    http::client::EspHttpConnection,
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use embedded_graphics::{
    mono_font::{MonoTextStyle, MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

use log::{error, info};

const SSID: &str = "Wokwi-GUEST";
const PASSWORD: &str = "";


fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let mut led = PinDriver::output(peripherals.pins.gpio15)?;
    
    let mut i2c_config = I2cConfig::default();
    i2c_config.baudrate = 100000.into();
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio22,
        peripherals.pins.gpio23,
        &i2c_config,
    )?;


    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    ).into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;
    log_info_and_display(&mut display, "Connecting to WiFi...", text_style.clone())?;
    display.flush().unwrap();

    connect_wifi(&mut wifi)?;
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    log_info_and_display(&mut display, &format!("Wifi DHCP info: {ip_info:?}"), text_style.clone())?;
    display.flush().unwrap();

    let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);

    // GET
    get_request(&mut client)?;
    info!("Shutting down in 5s...");

    std::thread::sleep(core::time::Duration::from_secs(5));

    let mut some_int: u8 = 0;
    loop {
        let _ = display.clear(BinaryColor::Off);
        let text: String = format!("Count: {}", &some_int);
        Text::with_baseline(&text, Point::new(0, 16), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        display.flush().unwrap();

        led.set_high()?;
        FreeRtos::delay_ms(50);
        led.set_low()?;
        FreeRtos::delay_ms(50);
        some_int = some_int.wrapping_add(1);
    }
}

/// Send an HTTP GET request.
fn get_request(client: &mut HttpClient<EspHttpConnection>) -> anyhow::Result<()> {
    // Prepare headers and URL
    let headers = [("accept", "text/plain")];
    let url = "http://ifconfig.net/";

    // Send request
    //
    // Note: If you don't want to pass in any headers, you can also use `client.get(url, headers)`.
    let request = client.request(Method::Get, url, &headers)?;
    info!("-> GET {url}");
    let mut response = request.submit()?;

    // Process response
    let status = response.status();
    info!("<- {status}");
    let mut buf = [0u8; 1024];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
    info!("Read {bytes_read} bytes");
    match std::str::from_utf8(&buf[0..bytes_read]) {
        Ok(body_string) => info!(
            "Response body (truncated to {} bytes): {:?}",
            buf.len(),
            body_string
        ),
        Err(e) => error!("Error decoding response body: {e}"),
    };

    Ok(())
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::None,
        password: PASSWORD.try_into().unwrap(),
        channel: None,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start()?;
    info!("Wifi started");

    wifi.connect()?;
    info!("Wifi connected");

    wifi.wait_netif_up()?;
    info!("Wifi netif up");

    Ok(())
}

fn log_info_and_display<'a, D>(
    display: &mut D,
    message: &str,
    text_style: MonoTextStyle<'a, BinaryColor>,
) -> anyhow::Result<()>
where
    D: embedded_graphics::prelude::DrawTarget<Color = BinaryColor>,
{
    info!("{}", message);
    let _ = display.clear(BinaryColor::Off);
    Text::with_baseline(message, Point::new(0, 0), text_style, Baseline::Top)
        .draw(display)
        .map_err(|_| anyhow::anyhow!("draw error"))?;
    Ok(())
}