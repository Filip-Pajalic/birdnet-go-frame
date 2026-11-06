//! Firmware for the Seeed XIAO EE02 (ESP32-S3) and the 13.3" Spectra 6 panel.
//!
//! Each wake it reads the battery and the buttons, joins the Wi-Fi (straight to the last access
//! point and address when it can), asks the frame server for `/panel` with the ETag of the page
//! on the glass, unpacks and checks a new page, draws it, installs new firmware if the server
//! offers it, and deep-sleeps for as long as the server's `Cache-Control: max-age` says or until
//! a button is pressed. Without settings, or with the refresh button held at power-up, it opens
//! a setup hotspot instead.
//!
//! The decisions are `panel-wake`'s and `panel-setup`'s, the radio is `transport-esp`'s and the
//! glass is `glass-esp`'s; this binary wires them to the board, which the `board-ee02`
//! (default) or `board-e1002` feature picks (the reTerminal E1002 and its 7.3" panel). See
//! `docs/how-to/flash-ee02-thin.md` and `docs/reference/firmware.md`.

#![no_std]
#![no_main]

extern crate alloc;

mod battery;
mod board;
mod buttons;
mod fetch;
mod memory;
mod settings;
mod setup;
mod update;

use alloc::vec;
use alloc::vec::Vec;

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_bootloader_esp_idf::partitions::FlashStorage;
use esp_hal::clock::CpuClock;
use esp_hal::psram::{PsramConfig, PsramMode};
use esp_hal::rng::Rng;
use esp_hal::rtc_cntl::sleep::{LowPower, RtcSleepConfig};
use esp_hal::time::{Duration, Instant};
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use panel_setup::FrameSettings;
use panel_wake::{SleepLimits, SleepMemory, Wake};

use crate::buttons::{Asked, Buttons};
use crate::fetch::{Fetched, Offer};
use crate::setup::Outcome;

// The descriptor the ESP-IDF bootloader checks before it starts the app, and which an update's
// version and board are read from.
esp_bootloader_esp_idf::esp_app_desc!(
    env!("CARGO_PKG_VERSION"),
    board::PROJECT,
    esp_bootloader_esp_idf::BUILD_TIME,
    esp_bootloader_esp_idf::BUILD_DATE,
    esp_bootloader_esp_idf::ESP_IDF_COMPATIBLE_VERSION,
    esp_bootloader_esp_idf::MMU_PAGE_SIZE,
    0,
    u16::MAX,
    esp_bootloader_esp_idf::SECURE_VERSION
);

/// One wake, from reset to deep sleep.
#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);
    // The 960,000-byte page and any firmware download live in the EE02's 8 MB of octal PSRAM.
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
    println!("ee02: firmware {}", fetch::VERSION);

    let limits = SleepLimits::default();
    let mut flash = FlashStorage::new(peripherals.FLASH);
    let mut buttons: Buttons<'_> = board::buttons!(peripherals);
    let asked: Asked = buttons.asked();
    let battery: Option<u16> = board::battery!(peripherals);
    if let Some(millivolts) = battery {
        println!("ee02: battery {millivolts} mV");
    }
    let seed: u64 = (u64::from(Rng::new().random()) << 32) | u64::from(Rng::new().random());
    let mut memory: SleepMemory = memory::load();
    let saved: Option<FrameSettings> = settings::load(&mut flash);
    let mut power = LowPower::new(peripherals.LPWR);

    let settings: FrameSettings = match (saved, asked) {
        (Some(settings), asked) if asked != Asked::Setup => settings,
        (saved, _) => {
            let configured: bool = saved.is_some();
            match setup::run(peripherals.WIFI, seed, &mut flash, saved, battery).await {
                Outcome::Saved => esp_hal::system::software_reset(),
                // Settings are still there: start over and use them.
                Outcome::TimedOut if configured => esp_hal::system::software_reset(),
                // Nothing to use: sleep until a button is pressed.
                Outcome::TimedOut => {
                    buttons.arm();
                    println!("ee02: not set up; sleeping until a button is pressed");
                    power.sleep_deep(RtcSleepConfig::deep())
                }
            }
        }
    };
    if let Asked::Pressed(button) = asked {
        println!("ee02: {button:?} pressed");
        memory = memory.after_press(button);
    }
    println!("ee02: asking for the {} page", memory.view);

    let mut page: Vec<u8> = vec![0; board::PAGE_LEN];
    let Fetched {
        wake,
        rejoin,
        update,
    } = fetch::fetch(
        peripherals.WIFI,
        seed,
        &settings,
        memory,
        battery,
        &mut page,
        limits,
    )
    .await;
    let wake: Wake = if wake.show {
        let refresh = board::show!(peripherals, &page);
        if let Err(error) = &refresh.slept {
            // The picture is on the glass by now, so this only gets a log line.
            println!("ee02: putting the panel to sleep: {error}");
        }
        shown(refresh.shown, wake, memory, limits)
    } else {
        wake
    };
    if let Some(error) = &wake.error {
        println!("ee02: {error}");
    } else {
        // This firmware reached the server: if it is a fresh update, it is a good one.
        update::confirm(&mut flash);
    }
    drop(page);

    let mut kept = SleepMemory {
        wake: wake.state,
        rejoin,
        ..memory
    };
    if let Some(Offer { version, image }) = update {
        let installed: bool = match image {
            Some(image) => match update::install(&mut flash, &image) {
                Ok(()) => true,
                Err(error) => {
                    println!("ee02: installing firmware {version}: {error}");
                    false
                }
            },
            None => false,
        };
        if installed {
            println!("ee02: firmware {version} installed; restarting into it");
            memory::store(&kept);
            esp_hal::system::software_reset();
        }
        kept.update = kept.update.failed(&version);
    }

    memory::store(&kept);
    buttons.arm();
    println!("ee02: sleeping {} s", wake.sleep_seconds);
    power.set_wakeup_deadline(Instant::now() + Duration::from_secs(u64::from(wake.sleep_seconds)));
    power.sleep_deep(RtcSleepConfig::deep())
}

/// Returns `wake` when the page reached the glass, or the failure wake when it did not.
fn shown<E: core::fmt::Display>(
    refresh: Result<u32, E>,
    wake: Wake,
    memory: SleepMemory,
    limits: SleepLimits,
) -> Wake {
    match refresh {
        Ok(milliseconds) => {
            println!("ee02: refreshed in {milliseconds} ms");
            wake
        }
        Err(error) => {
            println!("ee02: {error}");
            memory.wake.after_panel_failure(limits)
        }
    }
}
