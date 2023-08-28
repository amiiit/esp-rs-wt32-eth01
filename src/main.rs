// use esp_idf_hal::peripherals::Peripherals;
use esp_idf_sys as _;
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
// use esp_idf_svc::errors;
// use esp_idf_svc::eventloop::EspSystemEventLoop;
// use std::net::Ipv4Addr;
use std::error::Error;
// // use embedded_svc::eth;
use esp_idf_hal::gpio;

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;


fn main() -> Result<(), Box<dyn Error>>{
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let pins = peripherals.pins;

    info!("peripherals and pins"); // I can see this line logged

    // Seems like the following line is causing the runtime error:
    // E (627) esp.emac: emac_esp32_init(349): reset timeout
    // E (627) esp_eth: esp_eth_driver_install(214): init mac failed
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
                                                       esp_idf_svc::eth::RmiiEthChipset::LAN87XX,
                                                       None,
                                                       sysloop.clone(), )?;

    info!("created driver"); // this line I can't see logged anymore
    // Just:
    // I (627) esp_idf_svc::eventloop: System event loop dropped
    // Error: EspError(263)


    Ok(())

}
