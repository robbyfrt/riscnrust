use embedded_svc::{
    http::{client::Client as HttpClient, Method},
    utils::io};

use esp_idf_hal::{
    gpio::PinDriver,
    i2c::{I2cDriver, I2cConfig},
    peripherals::Peripherals,
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
    timer::EspTimerService,
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use embedded_graphics::{
    mono_font::{MonoTextStyle, MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
    primitives::{Rectangle,PrimitiveStyle}
}; 

use log::{info};

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASSWORD");


fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();
    let timer_service = EspTimerService::new()?;

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
        DisplayRotation::Rotate180
    ).into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    led.set_low()?;
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;
    log_info_and_display(&mut display, "Connecting to WiFi...", text_style.clone())?;
    display.flush().unwrap();

    connect_wifi(&mut wifi)?;
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    led.set_high()?;
    log_info_and_display(&mut display, &format!("Wifi DHCP info: {ip_info:?}"), text_style.clone())?;
    display.flush().unwrap();

    let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);

    // GET
    let result = get_request(&mut client)?;
    log_info_and_display(&mut display, &format!("GET: {}", &result), text_style.clone())?;
    display.flush().unwrap();


    std::thread::sleep(core::time::Duration::from_secs(5));

    let _ = display.clear(BinaryColor::Off);
    let mut some_int: u32 = 0;
    let mut now = timer_service.now();
    loop {
        let time_taken =  timer_service.now();
        let rssi = &wifi.wifi().get_rssi()?;
        update_line(&mut display, 0, &format!("wifi: {}dB", &rssi), text_style.clone())?;
        update_line(&mut display, 1, &format!("cnt: {}, t: {}ms", &some_int, (time_taken - now).as_millis()), text_style.clone())?;
        display.flush().unwrap();
        
        now = time_taken;
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
    for (i,line) in wrap_text(message, (128, 64), (6, 10)).iter().enumerate() {

        Text::with_baseline(line, Point::new(0, (i * 10) as i32), text_style, Baseline::Top)
            .draw(display)
            .map_err(|_| anyhow::anyhow!("draw error"))?;
    }
    Ok(())
}

fn update_line<'a, D>(
    display: &mut D,
    line_number: usize,
    message: &str,
    text_style: MonoTextStyle<'a, BinaryColor>,
) -> anyhow::Result<()>
where
    D: embedded_graphics::prelude::DrawTarget<Color = BinaryColor>,
{
    let _ = Rectangle::new(
        Point::new(0, (line_number * 10) as i32),
        Size::new(128, 10),
    )
    .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
    .draw(display)
    .map_err(|_| anyhow::anyhow!("draw error"))?;

    Text::with_baseline(
        message,
        Point::new(0, (line_number * 10) as i32),
        text_style,
        Baseline::Top,
    )
    .draw(display)
    .map_err(|_| anyhow::anyhow!("draw error"))?;
    Ok(())
}

fn wrap_text(text: &str, display_size: (usize,usize), font_size: (usize,usize)) ->  Vec<&str> {
    let   display_width_chars = display_size.0 / font_size.0;
    let _display_height_chars = display_size.1 / font_size.1;

    text.as_bytes()
        .chunks(display_width_chars)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect()
}