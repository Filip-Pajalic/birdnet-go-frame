//! The radio part of a wake: join, ask for the page (and new firmware, if offered), let go of
//! the radio.

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use embassy_futures::select::{self, Either};
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::Duration;
use esp_hal::peripherals::WIFI;
use esp_println::println;
use esp_radio::wifi::{ControllerConfig, Interface, WifiController};
use frame_core::PanelSize;
use panel_setup::FrameSettings;
use panel_wake::{
    ChipId, FirmwareImage, FrameAddress, PageRequest, Rejoin, ResponseHead, SleepLimits,
    SleepMemory, Wake,
};
use transport_esp::{Joined, TransportError, WifiCredentials};

/// How long a join to one access point on a known channel may take.
const REJOIN_TIMEOUT: Duration = Duration::from_secs(4);
/// How long a join that scans every channel may take.
const JOIN_TIMEOUT: Duration = Duration::from_secs(15);
/// How long DHCP may take.
const ADDRESS_TIMEOUT: Duration = Duration::from_secs(10);
/// How long the connection may stall.
const IDLE_TIMEOUT: Duration = Duration::from_secs(15);
/// The longest response kept: a page that does not compress at all, plus its head.
const RESPONSE_LIMIT: usize = 1_000_000;
/// The longest firmware kept: one app slot in `partitions.csv`, plus the response's head.
const FIRMWARE_LIMIT: usize = 0x3c_0000 + 1_024;
/// The firmware's name in its application description, which a download must match.
const PROJECT: &str = env!("CARGO_PKG_NAME");
/// The running firmware's version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// What the radio part of a wake decided.
pub struct Fetched {
    /// What to do: draw `page` or not, and how long to sleep.
    pub wake: Wake,
    /// How to join quickly next time, if this wake reached the frame.
    pub rejoin: Option<Rejoin>,
    /// The firmware the server offered and this wake tried to download.
    pub update: Option<Offer>,
}

/// Firmware the server offered.
pub struct Offer {
    /// The offered version.
    pub version: String,
    /// The checked image, or `None` when the download failed or did not pass the checks.
    pub image: Option<Vec<u8>>,
}

/// Joins the Wi-Fi, fast if `memory` knows how, asks the server for the page and unpacks a new
/// one into `page`. When the server offers other firmware and `memory` says to try it, downloads
/// that too. The radio is off again when this returns.
pub async fn fetch(
    wifi: WIFI<'_>,
    seed: u64,
    settings: &FrameSettings,
    memory: SleepMemory,
    battery_millivolts: Option<u16>,
    page: &mut [u8],
    limits: SleepLimits,
) -> Fetched {
    let exchanged: Result<Exchanged, TransportError> =
        exchange(wifi, seed, settings, memory, battery_millivolts).await;
    let Exchanged {
        response,
        rejoin,
        update,
    } = match exchanged {
        Ok(exchanged) => exchanged,
        Err(error) => {
            println!("ee02: {error}");
            return Fetched {
                wake: memory.wake.after_network_failure(limits),
                rejoin: None,
                update: None,
            };
        }
    };
    let Some((head, body)) = ResponseHead::split(&response) else {
        println!("ee02: the frame's answer has no HTTP head");
        return Fetched {
            wake: memory.wake.after_network_failure(limits),
            rejoin: Some(rejoin),
            update: None,
        };
    };
    println!(
        "ee02: {} with {} bytes, max-age {:?}",
        head.status,
        body.len(),
        head.max_age
    );
    Fetched {
        wake: memory.wake.after_response(&head, body, page, limits),
        rejoin: Some(rejoin),
        update,
    }
}

/// What came over the radio.
struct Exchanged {
    response: Vec<u8>,
    rejoin: Rejoin,
    update: Option<Offer>,
}

/// Turns the radio on, gets the response (and firmware, if wanted), and turns the radio off by
/// dropping it.
async fn exchange(
    wifi: WIFI<'_>,
    seed: u64,
    settings: &FrameSettings,
    memory: SleepMemory,
    battery_millivolts: Option<u16>,
) -> Result<Exchanged, TransportError> {
    let credentials = WifiCredentials {
        ssid: &settings.ssid,
        password: &settings.password,
    };
    let mut controller: WifiController<'_> = WifiController::new(wifi, ControllerConfig::default())
        .map_err(|_| TransportError::Radio)?;
    let (joined, rejoin): (Joined, Option<Rejoin>) =
        join(&mut controller, credentials, memory.rejoin).await?;
    // Boxed: the resources are too large for the stack.
    let mut resources: Box<StackResources<2>> = Box::new(StackResources::new());
    let (stack, mut runner): (Stack<'_>, Runner<'_, Interface>) = embassy_net::new(
        Interface::station(),
        transport_esp::network_config(rejoin.as_ref()),
        &mut resources,
        seed,
    );
    let server: FrameAddress = settings.server;
    let host: String = server.to_string();
    let request: String = PageRequest {
        rotation: settings.rotation,
        etag: memory.wake.etag(),
        view: memory.view,
        frame: Some(settings.frame.as_str()),
        battery_millivolts,
        firmware: Some(VERSION),
        ..PageRequest::new(&host, PanelSize::Inches13Point3)
    }
    .to_http();
    let work = async {
        transport_esp::wait_for_address(stack, ADDRESS_TIMEOUT).await?;
        let mut response: Vec<u8> = Vec::new();
        transport_esp::get(
            stack,
            server,
            request.as_bytes(),
            &mut response,
            RESPONSE_LIMIT,
            IDLE_TIMEOUT,
        )
        .await?;
        let rejoin: Rejoin = transport_esp::rejoin(joined, stack).ok_or(TransportError::Address)?;
        let offered: Option<String> =
            ResponseHead::split(&response).and_then(|(head, _)| head.firmware);
        let update: Option<Offer> = match offered {
            Some(version)
                if memory
                    .update
                    .wanted(VERSION, Some(&version), battery_millivolts) =>
            {
                println!("ee02: the server offers firmware {version}; downloading");
                let image: Option<Vec<u8>> = download(stack, server, &host, &version).await;
                Some(Offer { version, image })
            }
            _ => None,
        };
        Ok(Exchanged {
            response,
            rejoin,
            update,
        })
    };
    match select::select(runner.run(), work).await {
        Either::First(never) => never,
        Either::Second(result) => result,
    }
}

/// Downloads and checks the firmware the server offers as `version`. Returns the image without
/// its padding, or `None` after logging why not.
async fn download(
    stack: Stack<'_>,
    server: FrameAddress,
    host: &str,
    version: &str,
) -> Option<Vec<u8>> {
    let request: String = panel_wake::firmware_request(host);
    let mut response: Vec<u8> = Vec::new();
    if let Err(error) = transport_esp::get(
        stack,
        server,
        request.as_bytes(),
        &mut response,
        FIRMWARE_LIMIT,
        IDLE_TIMEOUT,
    )
    .await
    {
        println!("ee02: firmware download: {error}");
        return None;
    }
    let (head, body) = ResponseHead::split(&response)?;
    if head.status != 200 || !head.is_complete(body) {
        println!(
            "ee02: firmware download: status {}, incomplete or unsized",
            head.status
        );
        return None;
    }
    let length: usize = match FirmwareImage::check(body, ChipId::ESP32_S3, PROJECT) {
        Ok(image) if image.version == version => image.len,
        Ok(image) => {
            println!(
                "ee02: the server offered {version} but sent {}",
                image.version
            );
            return None;
        }
        Err(error) => {
            println!("ee02: firmware download: {error}");
            return None;
        }
    };
    let start: usize = response.len() - body.len();
    response.drain(..start);
    response.truncate(length);
    Some(response)
}

/// Joins straight to the remembered access point, or scans when there is none or it has moved.
/// Returns the access point and the record that worked, if any, for the address lease.
async fn join(
    controller: &mut WifiController<'_>,
    credentials: WifiCredentials<'_>,
    remembered: Option<Rejoin>,
) -> Result<(Joined, Option<Rejoin>), TransportError> {
    if let Some(rejoin) = remembered {
        match transport_esp::join(controller, credentials, Some(&rejoin), REJOIN_TIMEOUT).await {
            Ok(joined) => return Ok((joined, Some(rejoin))),
            Err(error) => println!("ee02: fast rejoin failed ({error}), scanning"),
        }
    }
    let joined: Joined = transport_esp::join(controller, credentials, None, JOIN_TIMEOUT).await?;
    Ok((joined, None))
}
