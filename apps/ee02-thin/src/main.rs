//! Firmware for the Seeed XIAO EE02 (ESP32-S3) and the 13.3" Spectra 6 panel.
//!
//! Each wake it joins the Wi-Fi (straight to the last access point and address when it can),
//! asks the frame for `/panel` with the ETag of the page on the glass, unpacks and checks a new
//! page, draws it, and deep-sleeps for as long as the frame's `Cache-Control: max-age` says. The
//! decisions are `panel-wake`'s, the radio is `transport-esp`'s and the glass is
//! `panel-t133a01`'s; this binary wires them to the board.
//!
//! Build-time settings: `FRAME_WIFI_SSID`, `FRAME_WIFI_PASSWORD`, `FRAME_SERVER` and optionally
//! `FRAME_ROTATION`. See `docs/how-to/flash-ee02-thin.md`.

#![no_std]
#![no_main]

extern crate alloc;

mod fetch;
mod glass;
mod memory;
mod settings;

use alloc::vec;
use alloc::vec::Vec;

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::psram::{PsramConfig, PsramMode};
use esp_hal::rng::Rng;
use esp_hal::rtc_cntl::sleep::{LowPower, RtcSleepConfig};
use esp_hal::time::{Duration, Instant};
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use panel_wake::{SleepLimits, SleepMemory, Wake};

use crate::fetch::Fetched;
use crate::glass::GlassPins;

// The descriptor the ESP-IDF bootloader checks before it starts the app.
esp_bootloader_esp_idf::esp_app_desc!();

/// One wake, from reset to deep sleep.
#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);
    // The 960,000-byte page lives in the EE02's 8 MB of octal PSRAM.
    esp_alloc::psram_allocator!(
        peripherals.PSRAM,
        esp_hal::psram,
        PsramConfig {
            mode: PsramMode::OctalSpi,
            ..PsramConfig::default()
        }
    );
    let timers = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timers.timer0, peripherals.FROM_CPU_INTR0);

    let limits = SleepLimits::default();
    let memory: SleepMemory = memory::load();
    let mut page: Vec<u8> = vec![0; panel_t133a01::PAGE_LEN];
    let seed: u64 = (u64::from(Rng::new().random()) << 32) | u64::from(Rng::new().random());

    let Fetched { wake, rejoin } = match settings::frame() {
        Ok(frame) => fetch::fetch(peripherals.WIFI, seed, frame, memory, &mut page, limits).await,
        Err(error) => {
            println!("ee02: FRAME_SERVER: {error}");
            Fetched {
                wake: memory.wake.after_network_failure(limits),
                rejoin: None,
            }
        }
    };
    let wake: Wake = if wake.show {
        let pins = GlassPins {
            spi: peripherals.SPI2,
            sck: peripherals.GPIO7,
            mosi: peripherals.GPIO9,
            cs0: peripherals.GPIO44,
            cs1: peripherals.GPIO41,
            dc: peripherals.GPIO10,
            rst: peripherals.GPIO38,
            busy: peripherals.GPIO4,
            power: peripherals.GPIO43,
        };
        match glass::show(pins, &page) {
            Ok(milliseconds) => {
                println!("ee02: refreshed in {milliseconds} ms");
                wake
            }
            Err(error) => {
                println!("ee02: {error}");
                memory.wake.after_panel_failure(limits)
            }
        }
    } else {
        wake
    };
    if let Some(error) = &wake.error {
        println!("ee02: {error}");
    }

    memory::store(&SleepMemory {
        wake: wake.state,
        rejoin,
    });
    println!("ee02: sleeping {} s", wake.sleep_seconds);
    let mut power = LowPower::new(peripherals.LPWR);
    power.set_wakeup_deadline(Instant::now() + Duration::from_secs(u64::from(wake.sleep_seconds)));
    power.sleep_deep(RtcSleepConfig::deep())
}
