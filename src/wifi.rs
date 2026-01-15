use esp_idf_svc::{
    wifi::{BlockingWifi, EspWifi, Configuration, ClientConfiguration, AuthMethod},
    nvs::EspDefaultNvsPartition,
    eventloop::EspSystemEventLoop,
};
use esp_idf_hal::modem::Modem;
use log::info;

pub struct WifiManager {
    wifi: BlockingWifi<EspWifi<'static>>,
}

impl WifiManager {
    pub fn new(
        sys_loop: EspSystemEventLoop,
        modem: Modem,
        nvs: EspDefaultNvsPartition,
        ssid: &'static str,
        password: &'static str,
    ) -> anyhow::Result<Self> {
        let mut wifi = BlockingWifi::wrap(
            EspWifi::new(modem, sys_loop.clone(), Some(nvs))?,
            sys_loop,
        )?;
        
        let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
            ssid: ssid.try_into().unwrap(),
            bssid: None,
            auth_method: AuthMethod::None,
            password: password.try_into().unwrap(),
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

        Ok(Self { wifi })
    }
    pub fn get_ip_info(&mut self) -> anyhow::Result<String> {
        let ip_info = self.wifi.wifi().sta_netif().get_ip_info()?;
        Ok(format!("{ip_info:?}"))
    }
    pub fn get_signal_strength(&mut self, as_bars: bool) -> anyhow::Result<i32> {
        let strength = self.wifi.wifi().get_rssi()?;
        if as_bars {
            let bars = match strength {
                s if s >= -50 => 5,
                s if s >= -60 => 4,
                s if s >= -70 => 3,
                s if s >= -80 => 2,
                s if s >= -90 => 1,
                _ => 0,
            };
            Ok(bars)
        } else {
        Ok(strength)
        }
    }
}