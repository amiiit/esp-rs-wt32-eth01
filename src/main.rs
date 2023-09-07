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
use esp_idf_svc::eth::{BlockingEth, EspEth};
use esp_idf_svc::eventloop::EspSystemEventLoop;

use embedded_svc::{
    http::{client::Client as HttpClient, Method},
    io::Write,
    utils::io,
};
use std::io::prelude::*;
use std::net::TcpStream;
use esp_idf_svc::http::client::EspHttpConnection;

fn test_https_client(client: &mut HttpClient<EspHttpConnection>) -> Result<(), Box<dyn Error>> {
    use embedded_svc::http::{self, client::*, status, Headers, Status};
    use embedded_svc::io::Read;
    use embedded_svc::utils::io;
    use esp_idf_svc::http::client::*;

    let url = String::from("https://google.com");

    info!("About to fetch content from {}", url);

    let mut response = client.get(&url)?.submit()?;

    let mut body = [0_u8; 1024];

    let read = io::try_read_full(&mut response, &mut body).map_err(|err| err.0)?;

    info!(
        "Body (truncated to 3K):\n{:?}",
        String::from_utf8_lossy(&body[..read]).into_owned()
    );

    // Complete the response
    while response.read(&mut body)? > 0 {}

    Ok(())
}

/// Send a HTTP GET request.
fn get_request(client: &mut HttpClient<EspHttpConnection>) -> Result<(), Box<dyn Error>> {
    // Prepare headers and URL
    let headers = [("accept", "text/plain"), ("connection", "close")];
    let url = "http://ifconfig.net/";

    // Send request
    //
    // Note: If you don't want to pass in any headers, you can also use `client.get(url, headers)`.
    let request = client.request(Method::Get, url, &headers)?;
    info!("-> GET {}", url);
    let mut response = request.submit()?;

    // Process response
    let status = response.status();
    info!("<- {}", status);
    let (_headers, mut body) = response.split();
    let mut buf = [0u8; 1024];
    let bytes_read = io::try_read_full(&mut body, &mut buf).map_err(|e| e.0)?;
    info!("Read {} bytes", bytes_read);
    match std::str::from_utf8(&buf[0..bytes_read]) {
        Ok(body_string) => info!(
            "Response body (truncated to {} bytes): {:?}",
            buf.len(),
            body_string
        ),
        Err(e) => error!("Error decoding response body: {}", e),
    };

    // Drain the remaining response bytes
    while body.read(&mut buf)? > 0 {}

    Ok(())
}

fn start_eth() -> Result<BlockingEth<EspEth<'static, esp_idf_svc::eth::RmiiEth>>, Box<dyn Error>> {
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
                                                       Some(pins.gpio16),
                                                       esp_idf_svc::eth::RmiiEthChipset::LAN87XX,
                                                       Some(1),
                                                       sysloop.clone(), )?;

    let eth = EspEth::wrap(driver)?;
    let mut eth = BlockingEth::wrap(eth, sysloop.clone())?;
    eth.start()?;
    info!("Waiting for DHCP lease...");
    eth.wait_netif_up()?;

    let ip_info = eth.eth().netif().get_ip_info()?;
    info!("Eth DHCP info: {:?}", ip_info);
    Ok(eth)
}

fn main() -> Result<(), Box<dyn Error>> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    info!("Start");
    let eth = start_eth()?;
    let mut client = HttpClient::wrap(EspHttpConnection::new(&esp_idf_svc::http::client::Configuration {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        buffer_size: Some(64000),
        ..Default::default()
    })?);

    loop {
        let _ = get_request(&mut client);
        let _ = test_https_client(&mut client);
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
