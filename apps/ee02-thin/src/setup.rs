//! The setup hotspot: the frame's own Wi-Fi network and its one page.

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

use embassy_futures::select::{self, Either};
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_bootloader_esp_idf::partitions::FlashStorage;
use esp_hal::peripherals::WIFI;
use esp_println::println;
use esp_radio::wifi::{ControllerConfig, Interface, WifiController};
use panel_setup::setup_page::{self, PageInfo};
use panel_setup::{DhcpServer, FrameSettings, Method, SetupRequest};
use transport_esp::{Answer, ReplyTo};

use crate::settings;

/// How long the hotspot stays open without a save.
const OPEN_FOR: Duration = Duration::from_secs(10 * 60);
/// The longest request read: the form is a few hundred bytes.
const REQUEST_LIMIT: usize = 4 * 1024;
/// How long a browser's connection may stall.
const IDLE: Duration = Duration::from_secs(10);

/// How the setup ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// New settings are in flash; restart to use them.
    Saved,
    /// Nobody saved anything in time, or the radio failed.
    TimedOut,
}

/// Opens the hotspot `BirdFrame-xxxx` with the setup page for ten minutes, or until settings
/// are saved. `current` fills in the form.
pub async fn run(
    wifi: WIFI<'_>,
    seed: u64,
    flash: &mut FlashStorage<'_>,
    current: Option<FrameSettings>,
    battery_millivolts: Option<u16>,
) -> Outcome {
    let Ok(mut controller) = WifiController::new(wifi, ControllerConfig::default()) else {
        println!("ee02: the radio did not start");
        return Outcome::TimedOut;
    };
    let networks: Vec<String> = transport_esp::scan(&mut controller)
        .await
        .unwrap_or_default();
    let interface: Interface = Interface::access_point();
    let mac: [u8; 6] = interface.mac_address();
    let name: String = format!("BirdFrame-{:02x}{:02x}", mac[4], mac[5]);
    if let Err(error) = transport_esp::open_hotspot(&mut controller, &name) {
        println!("ee02: the hotspot did not open: {error}");
        return Outcome::TimedOut;
    }
    println!("ee02: join the Wi-Fi \"{name}\" and open http://192.168.4.1/");

    // Boxed: the resources are too large for the stack.
    let mut resources: Box<StackResources<4>> = Box::new(StackResources::new());
    let (stack, mut runner): (Stack<'_>, Runner<'_, Interface>) = embassy_net::new(
        interface,
        transport_esp::hotspot_config(),
        &mut resources,
        seed,
    );
    let firmware: &str = env!("CARGO_PKG_VERSION");
    let state: RefCell<Page<'_, '_>> = RefCell::new(Page {
        flash,
        current,
        networks,
        battery_millivolts,
        firmware,
        saved: false,
    });
    let mut dhcp = DhcpServer::new(transport_esp::HOTSPOT_ADDRESS);
    let serve = async {
        let dhcp_answers = transport_esp::serve_udp(stack, 67, ReplyTo::Broadcast(68), |message| {
            dhcp.answer(message)
        });
        let dns_answers = transport_esp::serve_udp(stack, 53, ReplyTo::Sender, |query| {
            panel_setup::dns::captive_answer(query, transport_esp::HOTSPOT_ADDRESS)
        });
        let page = transport_esp::serve_http(stack, 80, REQUEST_LIMIT, IDLE, |request| {
            state.borrow_mut().answer(request)
        });
        let _ = select::select3(dhcp_answers, dns_answers, page).await;
    };
    let work = select::select(serve, Timer::after(OPEN_FOR));
    match select::select(runner.run(), work).await {
        Either::First(never) => never,
        Either::Second(_) => {}
    }
    if state.borrow().saved {
        Outcome::Saved
    } else {
        println!("ee02: nobody saved settings; closing the hotspot");
        Outcome::TimedOut
    }
}

/// What the page knows and changes.
struct Page<'f, 'd> {
    flash: &'f mut FlashStorage<'d>,
    current: Option<FrameSettings>,
    networks: Vec<String>,
    battery_millivolts: Option<u16>,
    firmware: &'static str,
    saved: bool,
}

impl Page<'_, '_> {
    /// The answer to a browser's request read so far.
    fn answer(&mut self, request: &[u8]) -> Answer {
        let Some((head, body)) = SetupRequest::split(request) else {
            return if request.len() >= REQUEST_LIMIT {
                Answer::Reply(panel_setup::http_response(
                    "400 Bad Request",
                    "text/plain",
                    b"",
                ))
            } else {
                Answer::ReadMore
            };
        };
        if !head.is_complete(body) {
            return Answer::ReadMore;
        }
        match (head.method, head.path) {
            (Method::Get, "/") => Answer::Reply(self.form(None)),
            (Method::Post, "/save") => self.save(&body[..head.content_length]),
            _ => Answer::Reply(panel_setup::redirect_to_page()),
        }
    }

    /// Checks and saves the form, or shows it again with what is wrong.
    fn save(&mut self, form: &[u8]) -> Answer {
        let settings: FrameSettings = match FrameSettings::from_form(form, self.current.as_ref()) {
            Ok(settings) => settings,
            Err(error) => return Answer::Reply(self.form(Some(&format!("{error}")))),
        };
        if let Err(error) = settings::save(self.flash, &settings) {
            return Answer::Reply(self.form(Some(&format!("{error}"))));
        }
        println!("ee02: settings saved for \"{}\"", settings.ssid);
        self.saved = true;
        let page: String = setup_page::saved(&settings);
        Answer::ReplyAndStop(panel_setup::http_response(
            "200 OK",
            "text/html; charset=utf-8",
            page.as_bytes(),
        ))
    }

    /// The setup page, with `message` above the form.
    fn form(&self, message: Option<&str>) -> Vec<u8> {
        let page: String = setup_page::render(
            self.current.as_ref(),
            &PageInfo {
                firmware: self.firmware,
                battery_millivolts: self.battery_millivolts,
                networks: &self.networks,
                message,
            },
        );
        panel_setup::http_response("200 OK", "text/html; charset=utf-8", page.as_bytes())
    }
}
