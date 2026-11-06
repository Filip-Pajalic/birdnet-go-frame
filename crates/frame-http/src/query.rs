//! Reading a request's query.

/// Returns the value of the first `key=value` pair in `query` (the part after `?`), as sent.
/// A key given without `=` has the empty value.
#[must_use]
pub fn query_value<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    query.split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        (name == key).then_some(value)
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn finds_the_first_value() {
        assert_eq!(
            query_value("frame=hall&view=week&view=x", "view"),
            Some("week")
        );
        assert_eq!(query_value("debug&frame=hall", "debug"), Some(""));
        assert_eq!(query_value("", "frame"), None);
    }
}
