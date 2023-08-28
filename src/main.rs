use esp_idf_hal::peripherals::Peripherals;
use esp_idf_sys as _;
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
use esp_idf_svc::errors;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use std::net::Ipv4Addr;
use std::error::Error;
// use embedded_svc::eth;
use esp_idf_hal::gpio;

fn main() -> Result<(), Box<dyn Error>> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let pins = peripherals.pins;

    let driver = esp_idf_svc::eth::EthDriver::new_rmii(peripherals.mac,
                                                       pins.gpio25,
                                                       pins.gpio26,
                                                       pins.gpio27,
                                                       pins.gpio23,
                                                       pins.gpio22,
                                                       pins.gpio21,
                                                       pins.gpio19,
                                                       pins.gpio18,
                                                       esp_idf_svc::eth::RmiiClockConfig::<gpio::Gpio0, gpio::Gpio16, gpio::Gpio17>::Input(
                                                           pins.gpio0,
                                                       ),
                                                       Some(pins.gpio5),
                                                       esp_idf_svc::eth::RmiiEthChipset::IP101,
                                                       None,
                                                       sysloop.clone(), )?;

    let eth = esp_idf_svc::eth::EspEth::wrap(
        driver,
    )?;

    let mut blocking_eth = esp_idf_svc::eth::BlockingEth::wrap(eth, sysloop.clone())?;
    blocking_eth.start()?;
    blocking_eth.wait_netif_up()?;

    info!("Hello, world!");
    Ok(())
}
