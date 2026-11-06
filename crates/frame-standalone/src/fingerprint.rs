//! Whether the page would change.

use birdnet_model::SpeciesSummary;
use frame_artist::Layout;
use frame_core::Crc32;
use panel_wake::PageView;

/// A checksum of what a page shows: the day, the view and the birds, and their counts on the
/// pages that print them. Two summaries with the same fingerprint draw the same page, so a frame
/// that already shows it need not draw it again.
///
/// The day counts because an art folder may dress a bird in another plate each day, as the
/// server's redraws do.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Fingerprint(pub u32);

impl Fingerprint {
    /// Returns the fingerprint of `view` drawn on `day` (days since 1970) in `layout` from
    /// `lists`, the answers the page is drawn from. The species' names always count; their
    /// counts only on pages that print numbers: the list, the story and the season.
    #[must_use]
    pub fn of(day: i64, view: PageView, lists: &[&[SpeciesSummary]], layout: Layout) -> Self {
        let counts: bool =
            layout == Layout::List || matches!(view, PageView::Story | PageView::Season);
        let mut crc: Crc32 = Crc32::new();
        crc.update(&day.to_le_bytes());
        crc.update(&[u8::from(layout == Layout::List), view.to_byte()]);
        for species in lists {
            crc.update(b"\x1e");
            for row in *species {
                for part in row.scientific_name.as_str().split_whitespace() {
                    crc.update(part.as_bytes());
                    crc.update(b" ");
                }
                crc.update(b"\n");
                if counts {
                    crc.update(row.common_name.as_bytes());
                    crc.update(&row.count.to_le_bytes());
                }
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
            Fingerprint::of(1, PageView::Today, &[&one], collage),
            Fingerprint::of(1, PageView::Today, &[&more], collage)
        );
        assert_ne!(
            Fingerprint::of(1, PageView::Today, &[&one], collage),
            Fingerprint::of(1, PageView::Today, &[&other], collage)
        );
        assert_ne!(
            Fingerprint::of(1, PageView::Today, &[&one], collage),
            Fingerprint::of(2, PageView::Today, &[&one], collage)
        );
        assert_ne!(
            Fingerprint::of(1, PageView::Today, &[&one], collage),
            Fingerprint::of(1, PageView::Week, &[&one], collage)
        );
        assert_ne!(
            Fingerprint::of(1, PageView::Story, &[&one], collage),
            Fingerprint::of(1, PageView::Story, &[&more], collage)
        );
        assert_ne!(
            Fingerprint::of(1, PageView::Today, &[&one], Layout::List),
            Fingerprint::of(1, PageView::Today, &[&more], Layout::List)
        );
    }
}
