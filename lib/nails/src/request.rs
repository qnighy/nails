use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::slice;

use async_trait::async_trait;
use hyper::{Body, Method, Request};

use crate::response::ErrorResponse;

pub use nails_derive::Preroute;

#[async_trait]
pub trait Preroute: Sized {
    fn path_prefix_hint() -> &'static str {
        ""
    }
    fn match_path(method: &Method, path: &str) -> bool;

    // TODO: Request<Body> -> RoutableRequest
    async fn from_request(req: Request<Body>) -> Result<Self, ErrorResponse>;
}

pub trait FromPath: Sized {
    fn from_path(path_component: &str) -> Result<Self, ()>;

    fn matches(path_component: &str) -> bool {
        Self::from_path(path_component).is_ok()
    }
}

impl FromPath for String {
    fn from_path(path_component: &str) -> Result<Self, ()> {
        Ok(path_component.to_owned())
    }
    fn matches(_path_component: &str) -> bool {
        true
    }
}

macro_rules! from_path_int_matcher {
    ($($int:ty)*) => {
        $(
            impl FromPath for $int {
                fn from_path(path_component: &str) -> Result<Self, ()> {
                    path_component.parse::<$int>().map_err(|_| ())
                }
            }
        )*
    };
}
from_path_int_matcher!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128);

pub trait FromQuery: Sized {
    // TODO: Result
    fn from_query(values: &[String]) -> Result<Self, ()>;
}

impl<T> FromQuery for Vec<T>
where
    T: FromQuery,
{
    fn from_query(values: &[String]) -> Result<Self, ()> {
        values
            .iter()
            .map(|x| T::from_query(slice::from_ref(x)))
            .collect()
    }
}

impl<T> FromQuery for Option<T>
where
    T: FromQuery,
{
    fn from_query(values: &[String]) -> Result<Self, ()> {
        if values.is_empty() {
            Ok(None)
        } else {
            Ok(Some(T::from_query(values)?))
        }
    }
}

impl FromQuery for String {
    fn from_query(values: &[String]) -> Result<Self, ()> {
        if values.len() != 1 {
            return Err(());
        }
        Ok(values[0].clone())
    }
}

impl FromQuery for i32 {
    fn from_query(values: &[String]) -> Result<Self, ()> {
        if values.len() != 1 {
            return Err(());
        }
        values[0].parse().map_err(|_| ())
    }
}

// TODO: rails-like decoding
// TODO: consider less-allocation way to decode query
// TODO: handle illformed keys and values
pub fn parse_query(query: &str) -> HashMap<String, Vec<String>> {
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
#[cfg_attr(tarpaulin, skip)]
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
    fn test_from_path() {
        assert_eq!(String::from_path("%20あ"), Ok(S("%20あ")));
        assert_eq!(i32::from_path("20"), Ok(20));
        assert_eq!(i32::from_path("08"), Ok(8));
        assert_eq!(i32::from_path("-2"), Ok(-2));
        assert_eq!(i32::from_path(" 5"), Err(()));
        assert_eq!(i8::from_path("200"), Err(()));
        assert_eq!(u32::from_path("-1"), Err(()));

        assert!(String::matches("%20あ"));
        assert!(i32::matches("20"));
        assert!(i32::matches("08"));
        assert!(i32::matches("-2"));
        assert!(!i32::matches(" 5"));
        assert!(!i8::matches("200"));
        assert!(!u32::matches("-1"));
    }

    #[test]
    fn test_from_query() {
        assert_eq!(String::from_query(&[]), Err(()));
        assert_eq!(String::from_query(&[S("foo")]), Ok(S("foo")));
        assert_eq!(String::from_query(&[S("foo"), S("bar")]), Err(()));
        assert_eq!(Option::<String>::from_query(&[]), Ok(None));
        assert_eq!(
            Option::<String>::from_query(&[S("foo")]),
            Ok(Some(S("foo")))
        );
        assert_eq!(Option::<String>::from_query(&[S("foo"), S("bar")]), Err(()));
        assert_eq!(i32::from_query(&[S("42")]), Ok(42));
        assert_eq!(i32::from_query(&[S("42"), S("42")]), Err(()));
        assert_eq!(i32::from_query(&[S("4x2")]), Err(()));
        assert_eq!(i32::from_query(&[]), Err(()));
        assert_eq!(Vec::<i32>::from_query(&[]), Ok(vec![]));
        assert_eq!(Vec::<i32>::from_query(&[S("42")]), Ok(vec![42]));
        assert_eq!(
            Vec::<i32>::from_query(&[S("42"), S("42")]),
            Ok(vec![42, 42])
        );
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
