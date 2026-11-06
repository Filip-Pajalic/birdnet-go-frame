//! The radio part of a wake: join, ask for the page, let go of the radio.

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
use panel_wake::{FrameAddress, Rejoin, ResponseHead, SleepLimits, SleepMemory, Wake};
use transport_esp::{Joined, TransportError};

use crate::settings;

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

/// What the radio part of a wake decided.
pub struct Fetched {
    /// What to do: draw `page` or not, and how long to sleep.
    pub wake: Wake,
    /// How to join quickly next time, if this wake reached the frame.
    pub rejoin: Option<Rejoin>,
}

/// Joins the Wi-Fi, fast if `memory` knows how, asks `frame` for the page and unpacks a new one
/// into `page`. The radio is off again when this returns.
pub async fn fetch(
    wifi: WIFI<'_>,
    seed: u64,
    frame: FrameAddress,
    memory: SleepMemory,
    page: &mut [u8],
    limits: SleepLimits,
) -> Fetched {
    let response: Result<(Vec<u8>, Rejoin), TransportError> =
        exchange(wifi, seed, frame, memory).await;
    let (response, rejoin) = match response {
        Ok(answer) => answer,
        Err(error) => {
            println!("ee02: {error}");
            return Fetched {
                wake: memory.wake.after_network_failure(limits),
                rejoin: None,
            };
        }
    };
    let Some((head, body)) = ResponseHead::split(&response) else {
        println!("ee02: the frame's answer has no HTTP head");
        return Fetched {
            wake: memory.wake.after_network_failure(limits),
            rejoin: Some(rejoin),
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
    }
}

/// Turns the radio on, gets the response, and turns the radio off by dropping it.
async fn exchange(
    wifi: WIFI<'_>,
    seed: u64,
    frame: FrameAddress,
    memory: SleepMemory,
) -> Result<(Vec<u8>, Rejoin), TransportError> {
    let mut controller: WifiController<'_> = WifiController::new(wifi, ControllerConfig::default())
        .map_err(|_| TransportError::Radio)?;
    let (joined, rejoin): (Joined, Option<Rejoin>) = join(&mut controller, memory.rejoin).await?;
    // Boxed: the resources are too large for the stack.
    let mut resources: Box<StackResources<2>> = Box::new(StackResources::new());
    let (stack, mut runner): (Stack<'_>, Runner<'_, Interface>) = embassy_net::new(
        Interface::station(),
        transport_esp::network_config(rejoin.as_ref()),
        &mut resources,
        seed,
    );
    let request: String = panel_wake::page_request(
        &frame.to_string(),
        PanelSize::Inches13Point3,
        settings::rotation(),
        memory.wake.etag(),
    );
    let work = async {
        transport_esp::wait_for_address(stack, ADDRESS_TIMEOUT).await?;
        let mut response: Vec<u8> = Vec::new();
        transport_esp::get(
            stack,
            frame,
            request.as_bytes(),
            &mut response,
            RESPONSE_LIMIT,
            IDLE_TIMEOUT,
        )
        .await?;
        let rejoin: Rejoin = transport_esp::rejoin(joined, stack).ok_or(TransportError::Address)?;
        Ok((response, rejoin))
    };
    match select::select(runner.run(), work).await {
        Either::First(never) => never,
        Either::Second(result) => result,
    }
}

/// Joins straight to the remembered access point, or scans when there is none or it has moved.
/// Returns the access point and the record that worked, if any, for the address lease.
async fn join(
    controller: &mut WifiController<'_>,
    remembered: Option<Rejoin>,
) -> Result<(Joined, Option<Rejoin>), TransportError> {
    if let Some(rejoin) = remembered {
        match transport_esp::join(controller, settings::WIFI, Some(&rejoin), REJOIN_TIMEOUT).await {
            Ok(joined) => return Ok((joined, Some(rejoin))),
            Err(error) => println!("ee02: fast rejoin failed ({error}), scanning"),
        }
    }
    let joined: Joined =
        transport_esp::join(controller, settings::WIFI, None, JOIN_TIMEOUT).await?;
    Ok((joined, None))
}
