//! Installing new firmware into the other app slot, and confirming the running one.

use alloc::vec;
use alloc::vec::Vec;

use esp_bootloader_esp_idf::ota::{Ota, OtaImageState};
use esp_bootloader_esp_idf::partitions::{
    self, AppPartitionSubType, DataPartitionSubType, Error, FlashStorage, PARTITION_TABLE_MAX_LEN,
    PartitionEntry, PartitionTable, PartitionType,
};

/// Bytes written to and read back from flash at a time.
const CHUNK: usize = 4096;
/// The app slots in `partitions.csv`.
const SLOTS: usize = 2;

/// Writes `image`, already checked by `panel_wake::FirmwareImage`, into the app slot that is not
/// running, reads it back, and tells the bootloader to start it next. The caller restarts.
///
/// # Errors
///
/// Returns the partition library's [`Error`] when the board has no second slot, the image does
/// not fit, or flash refuses a write or reads back differently.
pub fn install(flash: &mut FlashStorage<'_>, image: &[u8]) -> Result<(), Error> {
    let mut buffer: [u8; PARTITION_TABLE_MAX_LEN] = [0; PARTITION_TABLE_MAX_LEN];
    let table: PartitionTable<'_> = partitions::read_partition_table(flash, &mut buffer)?;
    let booted: Option<PartitionEntry> = table.booted_partition()?;
    let target: AppPartitionSubType = match booted.map(|entry| entry.partition_type()) {
        Some(PartitionType::App(AppPartitionSubType::Ota0)) => AppPartitionSubType::Ota1,
        _ => AppPartitionSubType::Ota0,
    };
    let slot: PartitionEntry = table
        .find_partition(PartitionType::App(target))?
        .ok_or(Error::Invalid)?;
    if image.len() > slot.len() as usize {
        return Err(Error::OutOfBounds);
    }
    let otadata: PartitionEntry = table
        .find_partition(PartitionType::Data(DataPartitionSubType::Ota))?
        .ok_or(Error::Invalid)?;

    let mut region = slot.as_flash_region(flash);
    let length: u32 = image.len().next_multiple_of(CHUNK) as u32;
    region.erase(0, length)?;
    // On the heap: the task's stack is small.
    let mut chunk: Vec<u8> = vec![0; CHUNK];
    let mut back: Vec<u8> = vec![0; CHUNK];
    for (index, part) in image.chunks(CHUNK).enumerate() {
        let offset: u32 = (index * CHUNK) as u32;
        // Flash is written in words; the padding reads as erased flash.
        let padded: usize = part.len().next_multiple_of(4);
        chunk[..part.len()].copy_from_slice(part);
        chunk[part.len()..padded].fill(0xff);
        region.write(offset, &chunk[..padded])?;
        region.read(offset, &mut back[..padded])?;
        if back[..padded] != chunk[..padded] {
            return Err(Error::StorageError);
        }
    }

    let mut ota = Ota::new(otadata.as_flash_region(flash), SLOTS)?;
    ota.set_current_app_partition(target)?;
    ota.set_current_ota_state(OtaImageState::New)
}

/// Marks the running firmware as good, if the bootloader is waiting to hear that. A bootloader
/// built with rollback would otherwise go back to the old firmware on the next reset.
pub fn confirm(flash: &mut FlashStorage<'_>) {
    let mut buffer: [u8; PARTITION_TABLE_MAX_LEN] = [0; PARTITION_TABLE_MAX_LEN];
    let Ok(table) = partitions::read_partition_table(flash, &mut buffer) else {
        return;
    };
    let Ok(Some(otadata)) = table.find_partition(PartitionType::Data(DataPartitionSubType::Ota))
    else {
        return;
    };
    let Ok(mut ota) = Ota::new(otadata.as_flash_region(flash), SLOTS) else {
        return;
    };
    if matches!(
        ota.current_ota_state(),
        Ok(OtaImageState::New | OtaImageState::PendingVerify)
    ) && ota.set_current_ota_state(OtaImageState::Valid).is_err()
    {
        esp_println::println!("ee02: could not mark this firmware as good");
    }
}
