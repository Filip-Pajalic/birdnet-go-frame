//! The frame's settings: from its flash partition, or else from the build.

use alloc::vec;
use alloc::vec::Vec;

use esp_bootloader_esp_idf::partitions::{
    self, FlashRegion, FlashStorage, PARTITION_TABLE_MAX_LEN,
};
use esp_println::println;
use frame_core::Rotation;
use panel_setup::FrameSettings;
use panel_wake::FrameAddress;

/// The label of the partition that holds the settings record (see `partitions.csv`).
const PARTITION: &str = "frame";
/// Bytes of a flash sector, the smallest part that can be erased.
const SECTOR: u32 = 4096;

/// Why the settings could not be saved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub struct SaveError;

impl core::fmt::Display for SaveError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("the settings could not be written to flash")
    }
}

impl core::error::Error for SaveError {}

/// Returns the saved settings, or the ones built in with `FRAME_WIFI_SSID`,
/// `FRAME_WIFI_PASSWORD`, `FRAME_SERVER` and `FRAME_ROTATION`, or `None` when there are
/// neither and the frame needs setting up.
pub fn load(flash: &mut FlashStorage<'_>) -> Option<FrameSettings> {
    let mut record: Vec<u8> = vec![0; FrameSettings::RECORD_MAX];
    let saved: Option<FrameSettings> = with_partition(flash, |region| {
        region.read(0, &mut record).ok()?;
        FrameSettings::from_record(&record)
    })
    .flatten();
    saved.or_else(built_in)
}

/// Writes `settings` to the settings partition.
///
/// # Errors
///
/// Returns [`SaveError`] when there is no settings partition or flash refuses the write.
pub fn save(flash: &mut FlashStorage<'_>, settings: &FrameSettings) -> Result<(), SaveError> {
    let mut record: Vec<u8> = settings.to_record();
    // Flash is written in words; the padding reads as erased flash.
    record.resize(record.len().next_multiple_of(4), 0xff);
    with_partition(flash, |region| {
        region.erase(0, SECTOR).map_err(|_| SaveError)?;
        region.write(0, &record).map_err(|_| SaveError)
    })
    .ok_or(SaveError)?
}

/// Runs `job` on the settings partition. Returns `None` when the partition table has none,
/// e.g. on a board flashed without `partitions.csv`.
fn with_partition<R>(
    flash: &mut FlashStorage<'_>,
    job: impl FnOnce(&mut FlashRegion<'_, '_>) -> R,
) -> Option<R> {
    let mut table: [u8; PARTITION_TABLE_MAX_LEN] = [0; PARTITION_TABLE_MAX_LEN];
    let found = partitions::read_partition_table(flash, &mut table)
        .ok()?
        .iter()
        .find(|entry| entry.label_as_str() == PARTITION);
    let Some(entry) = found else {
        println!("ee02: no \"{PARTITION}\" partition; flash with partitions.csv to keep settings");
        return None;
    };
    let mut region: FlashRegion<'_, '_> = entry.as_flash_region(flash);
    Some(job(&mut region))
}

/// The settings given at build time, if there are any and they pass the checks.
fn built_in() -> Option<FrameSettings> {
    let ssid: &str = option_env!("FRAME_WIFI_SSID").filter(|ssid| !ssid.is_empty())?;
    let server: FrameAddress = match option_env!("FRAME_SERVER").unwrap_or("").parse() {
        Ok(server) => server,
        Err(error) => {
            println!("ee02: FRAME_SERVER: {error}");
            return None;
        }
    };
    let rotation: Option<Rotation> = option_env!("FRAME_ROTATION")
        .filter(|text| !text.trim().is_empty())
        .and_then(|text| text.parse().ok());
    let password: &str = option_env!("FRAME_WIFI_PASSWORD").unwrap_or("");
    let frame: &str = option_env!("FRAME_NAME").unwrap_or("");
    match FrameSettings::new(ssid, password, server, rotation, frame) {
        Ok(settings) => Some(settings),
        Err(error) => {
            println!("ee02: the built-in settings: {error}");
            None
        }
    }
}
