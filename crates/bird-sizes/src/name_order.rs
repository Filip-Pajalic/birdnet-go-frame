//! Comparing scientific names the way people write them.

use core::cmp::Ordering;

/// Compares two scientific names ignoring ASCII case, surrounding space, and how many spaces
/// separate the words.
pub(crate) fn compare(left: &str, right: &str) -> Ordering {
    letters(left).cmp(letters(right))
}

/// The name's words in lowercase, joined by single spaces.
fn letters(name: &str) -> impl Iterator<Item = u8> + '_ {
    let mut words = name.split_ascii_whitespace();
    let mut current: Option<core::str::Bytes<'_>> = words.next().map(str::bytes);
    let mut space_due: bool = false;
    core::iter::from_fn(move || {
        loop {
            if space_due {
                space_due = false;
                return Some(b' ');
            }
            let bytes = current.as_mut()?;
            if let Some(byte) = bytes.next() {
                return Some(byte.to_ascii_lowercase());
            }
            current = words.next().map(str::bytes);
            space_due = current.is_some();
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ignores_case_and_spacing() {
        assert_eq!(compare("Parus major", " parus   MAJOR "), Ordering::Equal);
        assert_eq!(compare("Parus major", "Parus majorx"), Ordering::Less);
        assert_eq!(compare("Pica pica", "Parus major"), Ordering::Greater);
        assert_eq!(compare("", "  "), Ordering::Equal);
    }
}
