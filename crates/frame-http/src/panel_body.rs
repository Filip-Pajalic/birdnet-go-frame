//! What `/panel` sends: the controllers' bytes, plain or compressed.

/// A page laid out as a panel's controllers take it, in both forms `/panel` sends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanelBody {
    /// The bytes as [`frame_core::panel_bytes::pack`] lays them out.
    pub plain: Vec<u8>,
    /// `plain` as a zlib stream (RFC 1950), sent with `Content-Encoding: deflate` to clients
    /// that accept it: about a quarter of the bytes, so the radio is on for less time.
    pub zlib: Vec<u8>,
}
