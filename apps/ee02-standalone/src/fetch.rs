//! The radio part of a wake: join, ask BirdNET-Go for the time and what the page shows, let go
//! of the radio.

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use birdnet_model::{SpeciesHours, SpeciesSummary};
use embassy_futures::select::{self, Either};
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::Duration;
use esp_hal::peripherals::WIFI;
use esp_println::println;
use esp_radio::wifi::{ControllerConfig, Interface, WifiController};
use frame_artist::PageView;
use frame_standalone::{AnswerError, BirdnetAnswer, Summary, ViewFetch};
use panel_wake::{FrameAddress, Rejoin};
use transport_esp::{Joined, TransportError, WifiCredentials};

use crate::settings::Settings;

/// How long a join to one access point on a known channel may take.
const REJOIN_TIMEOUT: Duration = Duration::from_secs(4);
/// How long a join that scans every channel may take.
const JOIN_TIMEOUT: Duration = Duration::from_secs(15);
/// How long DHCP may take.
const ADDRESS_TIMEOUT: Duration = Duration::from_secs(10);
/// How long BirdNET-Go may stall: a species summary over a year can take it a few seconds.
const IDLE_TIMEOUT: Duration = Duration::from_secs(20);
/// The longest answer kept. A summary is about 150 bytes a species, the year's included.
const RESPONSE_LIMIT: usize = 256 * 1024;
/// Where BirdNET-Go says how it is, with the time in its `Date` header.
const HEALTH_PATH: &str = "/api/v2/health";

/// What BirdNET-Go said.
pub struct Heard {
    /// The unix time BirdNET-Go's clock gave.
    pub now: i64,
    /// What the page is drawn from.
    pub fetched: ViewFetch,
    /// How to join quickly next time.
    pub rejoin: Rejoin,
}

/// Why a wake heard nothing.
#[derive(Debug)]
pub enum Failure {
    /// The Wi-Fi or the connection failed.
    Transport(TransportError),
    /// BirdNET-Go's answer gave no birds.
    Answer(AnswerError),
    /// BirdNET-Go's health answer had no `Date` header the frame can read.
    NoDate,
}

impl fmt::Display for Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(error) => write!(formatter, "{error}"),
            Self::Answer(error) => write!(formatter, "BirdNET-Go: {error}"),
            Self::NoDate => formatter.write_str("BirdNET-Go's answer has no Date header"),
        }
    }
}

impl From<TransportError> for Failure {
    fn from(error: TransportError) -> Self {
        Self::Transport(error)
    }
}

impl From<AnswerError> for Failure {
    fn from(error: AnswerError) -> Self {
        Self::Answer(error)
    }
}

/// Joins the Wi-Fi, fast if `remembered` says how, and asks BirdNET-Go for the time and then
/// for what page `view` shows (see [`ViewFetch`]). The radio is off again when this returns.
///
/// # Errors
///
/// Returns the [`Failure`] that stopped the wake.
pub async fn fetch(
    wifi: WIFI<'_>,
    seed: u64,
    settings: &Settings,
    remembered: Option<Rejoin>,
    view: PageView,
) -> Result<Heard, Failure> {
    let credentials = WifiCredentials {
        ssid: settings.ssid,
        password: settings.password,
    };
    let mut controller: WifiController<'_> = WifiController::new(wifi, ControllerConfig::default())
        .map_err(|_| TransportError::Radio)?;
    let (joined, rejoin): (Joined, Option<Rejoin>) =
        join(&mut controller, credentials, remembered).await?;
    // Boxed: the resources are too large for the stack.
    let mut resources: Box<StackResources<2>> = Box::new(StackResources::new());
    let (stack, mut runner): (Stack<'_>, Runner<'_, Interface>) = embassy_net::new(
        Interface::station(),
        transport_esp::network_config(rejoin.as_ref()),
        &mut resources,
        seed,
    );
    let work = async {
        transport_esp::wait_for_address(stack, ADDRESS_TIMEOUT).await?;
        let birdnet: FrameAddress = settings.frame.birdnet;
        let health: Vec<u8> = ask(stack, birdnet, HEALTH_PATH).await?;
        let now: i64 = BirdnetAnswer::parse(&health)?.date.ok_or(Failure::NoDate)?;
        drop(health);
        let mut fetched = ViewFetch::new(
            view,
            now,
            settings.frame.utc_offset_minutes,
            settings.frame.window_hours,
        );
        let summaries: Vec<(Summary, String)> = fetched.summaries().collect();
        for (summary, path) in summaries {
            let answer: Vec<u8> = ask(stack, birdnet, &path).await?;
            let species: Vec<SpeciesSummary> = BirdnetAnswer::parse(&answer)?.species()?;
            fetched.answer(summary, species);
        }
        for path in fetched.hourly_paths() {
            let answer: Vec<u8> = ask(stack, birdnet, &path).await?;
            let hours: Vec<SpeciesHours> = BirdnetAnswer::parse(&answer)?.hours()?;
            fetched.answer_hours(hours);
        }
        let rejoin: Rejoin = transport_esp::rejoin(joined, stack).ok_or(TransportError::Address)?;
        Ok(Heard {
            now,
            fetched,
            rejoin,
        })
    };
    match select::select(runner.run(), work).await {
        Either::First(never) => never,
        Either::Second(result) => result,
    }
}

/// Sends BirdNET-Go at `birdnet` a GET of `path` and returns the whole response.
async fn ask(stack: Stack<'_>, birdnet: FrameAddress, path: &str) -> Result<Vec<u8>, Failure> {
    let host: String = birdnet.to_string();
    let request: String = frame_standalone::birdnet_request(&host, path);
    let mut response: Vec<u8> = Vec::new();
    transport_esp::get(
        stack,
        birdnet,
        request.as_bytes(),
        &mut response,
        RESPONSE_LIMIT,
        IDLE_TIMEOUT,
    )
    .await?;
    Ok(response)
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
            Err(error) => println!("standalone: fast rejoin failed ({error}), scanning"),
        }
    }
    let joined: Joined = transport_esp::join(controller, credentials, None, JOIN_TIMEOUT).await?;
    Ok((joined, None))
}
