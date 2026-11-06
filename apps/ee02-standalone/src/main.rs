//! Firmware for an ESP32-S3 with a Spectra 6 panel that needs no frame server.
//!
//! Each wake it joins the Wi-Fi (straight to the last access point and address when it can),
//! asks BirdNET-Go for the time and for the birds heard in the last hours, and draws the page
//! itself when the birds changed: the same collage or list the server draws, at half size on
//! the 13.3" panel so it fits in PSRAM, with the art bundled into flash. Then it deep-sleeps
//! until the next poll.
//!
//! The decisions are `frame-standalone`'s, the drawing is `frame-artist`'s, the radio is
//! `transport-esp`'s and the glass is `glass-esp`'s; this binary wires them to the board, which
//! the `board-ee02` (default) or `board-e1002` feature picks. See
//! `docs/how-to/flash-ee02-standalone.md`.

#![no_std]
#![no_main]

extern crate alloc;

mod board;
mod draw;
mod fetch;
mod memory;
mod settings;

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
use frame_standalone::{StandaloneMemory, StandaloneWake};

use crate::fetch::Heard;
use crate::settings::Settings;

// The descriptor the ESP-IDF bootloader checks before it starts the app.
esp_bootloader_esp_idf::esp_app_desc!();

/// How long to sleep when the built-in settings cannot be read: they will not change until the
/// board is flashed again, so there is no hurry.
const UNSET_SLEEP_SECONDS: u32 = 3_600;

/// One wake, from reset to deep sleep.
#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);
    // The page, the pictures and the font live in the board's 8 MB of octal PSRAM.
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
    println!(
        "standalone: firmware {} for the {}",
        env!("CARGO_PKG_VERSION"),
        board::NAME
    );

    let mut power = LowPower::new(peripherals.LPWR);
    let settings: Settings = match settings::load() {
        Ok(settings) => settings,
        Err(problem) => {
            println!("standalone: {problem}; build again with the settings");
            sleep(&mut power, UNSET_SLEEP_SECONDS)
        }
    };
    let seed: u64 = (u64::from(Rng::new().random()) << 32) | u64::from(Rng::new().random());
    let memory: StandaloneMemory = memory::load();
    let poll_minutes: u16 = settings.frame.poll_minutes;

    let heard: Heard = match fetch::fetch(peripherals.WIFI, seed, &settings, memory.rejoin).await {
        Ok(heard) => heard,
        Err(problem) => {
            println!("standalone: {problem}");
            let (kept, seconds): (StandaloneMemory, u32) =
                memory.after_failure(poll_minutes, false);
            // A rejoin that led nowhere is forgotten: the next wake scans.
            let kept = StandaloneMemory {
                rejoin: None,
                ..kept
            };
            memory::store(&kept);
            sleep(&mut power, seconds)
        }
    };
    let memory = StandaloneMemory {
        rejoin: Some(heard.rejoin),
        ..memory
    };
    let day: i64 = draw::local_day(heard.now, settings.frame.utc_offset_minutes);
    let wake: StandaloneWake =
        memory.after_species(day, &heard.species, settings.frame.layout, poll_minutes);
    println!(
        "standalone: {} species heard; {}",
        heard.species.len(),
        if wake.show {
            "drawing"
        } else {
            "the page has not changed"
        }
    );
    if !wake.show {
        memory::store(&memory.after_shown(wake.fingerprint));
        sleep(&mut power, wake.sleep_seconds)
    }

    let started: Instant = Instant::now();
    let page: Vec<u8> = match draw::page(&settings.frame, board::PANEL, heard.now, &heard.species) {
        Ok(page) => page,
        Err(problem) => {
            println!("standalone: {problem}");
            let (kept, seconds) = memory.after_failure(poll_minutes, true);
            memory::store(&kept);
            sleep(&mut power, seconds)
        }
    };
    drop(heard);
    println!(
        "standalone: drew the page in {} ms",
        started.elapsed().as_millis()
    );
    let refresh = board::show!(peripherals, &page);
    drop(page);
    if let Err(problem) = &refresh.slept {
        println!("standalone: putting the panel to sleep: {problem}");
    }
    let (kept, seconds): (StandaloneMemory, u32) = match refresh.shown {
        Ok(milliseconds) => {
            println!("standalone: refreshed in {milliseconds} ms");
            (memory.after_shown(wake.fingerprint), wake.sleep_seconds)
        }
        Err(problem) => {
            println!("standalone: {problem}");
            memory.after_failure(poll_minutes, true)
        }
    };
    memory::store(&kept);
    sleep(&mut power, seconds)
}

/// Deep-sleeps for `seconds`; the next wake starts from reset.
fn sleep(power: &mut LowPower<'_>, seconds: u32) -> ! {
    println!("standalone: sleeping {seconds} s");
    power.set_wakeup_deadline(Instant::now() + Duration::from_secs(u64::from(seconds)));
    power.sleep_deep(RtcSleepConfig::deep())
}
