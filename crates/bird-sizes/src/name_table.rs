//! A CSV table keyed by scientific name, read in place.

use core::cmp::Ordering;

use crate::name_order;

/// A table whose first line is a header and whose other lines start with a scientific name and
/// a comma, sorted by name ignoring case.
///
/// Lookups are a binary search over the text's lines, so the table needs no allocation.
///
/// ```
/// use bird_sizes::NameTable;
///
/// let table = NameTable::parse_csv("name,mass\nParus major,16.3\nPica pica,217\n")
///     .expect("sorted");
/// assert_eq!(table.row("parus MAJOR"), Some("16.3"));
/// assert_eq!(table.row("Corvus corax"), None);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct NameTable<'a> {
    /// Data lines, without the header.
    rows: &'a str,
    line_count: usize,
}

impl<'a> NameTable<'a> {
    /// Reads `text`. Lines without a comma are ignored by lookups.
    ///
    /// Returns `None` if the lines are not sorted by name, ignoring case and spacing.
    #[must_use]
    pub fn parse_csv(text: &'a str) -> Option<Self> {
        let rows: &str = text.split_once('\n').map_or("", |(_header, rest)| rest);
        let mut previous: Option<&str> = None;
        let mut line_count: usize = 0;
        for line in rows.lines() {
            let Some((name, _rest)) = line.split_once(',') else {
                continue;
            };
            if previous.is_some_and(|before| name_order::compare(before, name).is_gt()) {
                return None;
            }
            previous = Some(name);
            line_count += 1;
        }
        Some(Self { rows, line_count })
    }

    /// Returns how many species the table has.
    #[must_use]
    pub fn len(&self) -> usize {
        self.line_count
    }

    /// Returns whether the table has no species.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.line_count == 0
    }

    /// Returns every row as its name and the fields after it, in order.
    pub fn rows(&self) -> impl Iterator<Item = (&'a str, &'a str)> + 'a {
        self.rows
            .lines()
            .filter_map(|line| line.trim_end().split_once(','))
    }

    /// Returns the fields after the name of the species with this scientific name, compared
    /// ignoring case and spacing, or `None` when the table does not know it.
    #[must_use]
    pub fn row(&self, scientific_name: &str) -> Option<&'a str> {
        // Binary search over byte offsets: jump to the middle, step to the next line start.
        let bytes: &[u8] = self.rows.as_bytes();
        let mut low: usize = 0;
        let mut high: usize = bytes.len();
        while low < high {
            let mut middle: usize = line_start(bytes, low + (high - low) / 2);
            if middle >= high {
                // No line starts in the upper half; step through the lower one from `low`.
                middle = low;
            }
            let (order, fields) = self.compare_line(middle, scientific_name);
            match order {
                Ordering::Equal => return fields,
                Ordering::Less => low = line_end(bytes, middle),
                Ordering::Greater => high = middle,
            }
        }
        None
    }

    /// Compares the line starting at `start` with `wanted`, returning its fields when equal.
    fn compare_line(&self, start: usize, wanted: &str) -> (Ordering, Option<&'a str>) {
        let end: usize = line_end(self.rows.as_bytes(), start);
        let line: &'a str = self.rows.get(start..end).unwrap_or("").trim_end();
        let Some((name, fields)) = line.split_once(',') else {
            return (Ordering::Less, None);
        };
        let order: Ordering = name_order::compare(name, wanted);
        (order, order.is_eq().then_some(fields))
    }
}

/// Returns the start of the first line at or after `index`.
fn line_start(bytes: &[u8], index: usize) -> usize {
    if index == 0 {
        return 0;
    }
    match bytes
        .get(index - 1..)
        .and_then(|rest| rest.iter().position(|&byte| byte == b'\n'))
    {
        Some(offset) => index + offset,
        None => bytes.len(),
    }
}

/// Returns the start of the line after the one starting at `start`.
fn line_end(bytes: &[u8], start: usize) -> usize {
    match bytes
        .get(start..)
        .and_then(|rest| rest.iter().position(|&byte| byte == b'\n'))
    {
        Some(offset) => start + offset + 1,
        None => bytes.len(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn finds_short_lines_hidden_behind_a_long_one() {
        let text = "h\nA a,1\nB b,2\nZzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz z,3\n";
        let table = NameTable::parse_csv(text).expect("sorted");
        assert_eq!(table.row("B b"), Some("2"));
        assert_eq!(table.row("A a"), Some("1"));
        assert_eq!(table.row("C c"), None);
        assert_eq!(table.rows().count(), 3);
    }

    #[test]
    fn rejects_unsorted_tables() {
        assert!(NameTable::parse_csv("h\nPica pica,1\nParus major,2\n").is_none());
        assert!(
            NameTable::parse_csv("")
                .expect("empty is sorted")
                .is_empty()
        );
    }
}
