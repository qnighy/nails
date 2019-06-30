use std::collections::hash_map::Entry;
use std::collections::HashMap;

use hyper::{Body, Method, Request};

pub use nails_derive::FromRequest;

pub trait FromRequest: Sized {
    fn path_prefix_hint() -> &'static str {
        ""
    }
    fn match_path(method: &Method, path: &str) -> bool;

    // TODO: Result<Self>
    // TODO: Request<Body> -> RoutableRequest
    fn from_request(req: Request<Body>) -> Self;
}

// TODO: rails-like decoding
// TODO: consider less-allocation way to decode query
// TODO: handle illformed keys and values
fn parse_query(query: &str) -> HashMap<String, Vec<String>> {
    let mut hash: HashMap<String, Vec<String>> = HashMap::new();
    for pair in query.split("&") {
        let (key, value) = if let Some(pair) = parse_query_pair(pair) {
            pair
        } else {
            // TODO: handle errors
            continue;
        };
        match hash.entry(key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(value);
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![value]);
            }
        }
    }
    hash
}

// TODO: optimize
// TODO: better error handling
fn parse_query_pair(pair: &str) -> Option<(String, String)> {
    let mut kv_iter = pair.split("=");
    let key = kv_iter.next().unwrap(); // Split always yields some element
    let value = kv_iter.next()?;
    if kv_iter.next().is_some() {
        return None;
    }
    let key = parse_percent_encoding(key)?;
    let value = parse_percent_encoding(value)?;
    Some((key, value))
}

// TODO: optimize
// TODO: better error handling
fn parse_percent_encoding(input: &str) -> Option<String> {
    let input = input.as_bytes();
    let mut output = Vec::new();
    let mut i = 0;
    while i < input.len() {
        if input[i] == b'%' {
            if i + 3 > input.len() {
                return None;
            }
            let d0 = input[i + 1];
            let d1 = input[i + 2];
            if !(d0.is_ascii_hexdigit() && d1.is_ascii_hexdigit()) {
                return None;
            }
            let d0 = (d0 as char).to_digit(16).unwrap();
            let d1 = (d1 as char).to_digit(16).unwrap();
            output.push((d0 * 16 + d1) as u8);
            i += 3;
        } else {
            output.push(input[i]);
            i += 1;
        }
    }
    String::from_utf8(output).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! hash {
        ($($e:expr),*) => {
            vec![$($e,)*].into_iter().collect::<std::collections::HashMap<_, _>>()
        };
        ($($e:expr,)*) => {
            vec![$($e,)*].into_iter().collect::<std::collections::HashMap<_, _>>()
        };
    }

    #[test]
    fn test_parse_query() {
        assert_eq!(parse_query("foo=bar"), hash![(S("foo"), vec![S("bar")])]);
        assert_eq!(
            parse_query("foo=bar&foo=baz"),
            hash![(S("foo"), vec![S("bar"), S("baz")])],
        );
        assert_eq!(
            parse_query("foo=bar&foo2=baz"),
            hash![(S("foo"), vec![S("bar")]), (S("foo2"), vec![S("baz")])],
        );
        assert_eq!(parse_query("foo&foo=foo=foo&f%oo=1&1=%E3"), hash![]);
    }

    #[test]
    fn test_parse_percent_encoding() {
        assert_eq!(parse_percent_encoding("foo"), Some(S("foo")));
        assert_eq!(parse_percent_encoding("f%6F%6f"), Some(S("foo")));
        assert_eq!(parse_percent_encoding("あ"), Some(S("あ")));
        assert_eq!(parse_percent_encoding("%E3%81%82"), Some(S("あ")));
    }

    #[test]
    fn test_parse_percent_encoding_failing() {
        assert_eq!(parse_percent_encoding("fo%"), None);
        assert_eq!(parse_percent_encoding("%kv"), None);
        assert_eq!(parse_percent_encoding("%E3%81"), None);
    }

    #[allow(non_snake_case)]
    fn S(s: &'static str) -> String {
        s.to_owned()
    }
}
