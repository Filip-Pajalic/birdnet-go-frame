//! Whether the page would change.

use birdnet_model::SpeciesSummary;
use frame_artist::Layout;
use frame_core::Crc32;

/// A checksum of what a page shows: the day and the birds, and their counts on the list page,
/// which prints them. Two summaries with the same fingerprint draw the same page, so a frame
/// that already shows it need not draw it again.
///
/// The day counts because an art folder may dress a bird in another plate each day, as the
/// server's redraws do.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Fingerprint(pub u32);

impl Fingerprint {
    /// Returns the fingerprint of `species` (most heard first) drawn as `layout` on `day` (days
    /// since 1970).
    #[must_use]
    pub fn of(day: i64, species: &[SpeciesSummary], layout: Layout) -> Self {
        let mut crc: Crc32 = Crc32::new();
        crc.update(&day.to_le_bytes());
        crc.update(&[u8::from(layout == Layout::List)]);
        for row in species {
            for part in row.scientific_name.as_str().split_whitespace() {
                crc.update(part.as_bytes());
                crc.update(b" ");
            }
            crc.update(b"\n");
            if layout == Layout::List {
                crc.update(row.common_name.as_bytes());
                crc.update(&row.count.to_le_bytes());
            }
        }
        Self(crc.finish())
    }
}

#[cfg(test)]
mod test {
    use alloc::vec::Vec;

    use super::*;

    fn summary(json: &[u8]) -> Vec<SpeciesSummary> {
        SpeciesSummary::parse_list(json).expect("valid JSON")
    }

    #[test]
    fn changes_with_the_birds_and_the_day_and_on_the_list_with_counts() {
        let one: Vec<SpeciesSummary> =
            summary(br#"[{"scientific_name":"Parus major","common_name":"Great Tit","count":3}]"#);
        let more: Vec<SpeciesSummary> =
            summary(br#"[{"scientific_name":"Parus major","common_name":"Great Tit","count":4}]"#);
        let other: Vec<SpeciesSummary> =
            summary(br#"[{"scientific_name":"Pica pica","common_name":"Magpie","count":3}]"#);
        let collage = Layout::Collage;
        assert_eq!(
            Fingerprint::of(1, &one, collage),
            Fingerprint::of(1, &more, collage)
        );
        assert_ne!(
            Fingerprint::of(1, &one, collage),
            Fingerprint::of(1, &other, collage)
        );
        assert_ne!(
            Fingerprint::of(1, &one, collage),
            Fingerprint::of(2, &one, collage)
        );
        assert_ne!(
            Fingerprint::of(1, &one, Layout::List),
            Fingerprint::of(1, &more, Layout::List)
        );
    }
}
