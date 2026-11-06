//! What every panel driver can do, so apps can be written once for any panel.

use crate::panel_size::PanelSize;

/// A Spectra 6 panel behind its driver: wake it, show a page, put it to sleep.
///
/// The page is the bytes [`crate::panel_bytes::pack`] lays out for [`Glass::PANEL`], as `/panel`
/// serves them once inflated.
pub trait Glass {
    /// Why the panel could not be driven.
    type Error: core::error::Error;

    /// The panel this driver drives.
    const PANEL: PanelSize;

    /// Resets the controllers and powers the panel on, ready for a page.
    ///
    /// # Errors
    ///
    /// Returns an error when the controllers cannot be reached or stay busy.
    fn wake(&mut self) -> Result<(), Self::Error>;

    /// Sends `page` and refreshes the glass with it. Returns how many milliseconds the refresh
    /// took.
    ///
    /// # Errors
    ///
    /// Returns an error when `page` is not one page for [`Glass::PANEL`], or the panel fails to
    /// take or show it.
    fn show(&mut self, page: &[u8]) -> Result<u32, Self::Error>;

    /// Powers the panel off. The glass keeps its picture with no power at all.
    ///
    /// # Errors
    ///
    /// Returns an error when the controllers cannot be reached or stay busy.
    fn sleep(&mut self) -> Result<(), Self::Error>;
}
