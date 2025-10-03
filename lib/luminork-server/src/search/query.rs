use std::str::FromStr;

use ulid::Ulid;

use crate::search::{
    Error,
    Result,
    parser,
};

/// A search query
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchQuery {
    MatchValue(SearchTerm),
    MatchAttr {
        name: String,
        terms: Vec<SearchTerm>,
    },
    And(Vec<SearchQuery>),
    /// One of the sub-queries must match
    Or(Vec<SearchQuery>),
    /// The sub-query must not match
    Not(Box<SearchQuery>),
    /// Used for the empty query. Matches everything.
    All,
}

/// A value to match
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchTerm {
    /// Match the string, allowing * and such: Instance, AWS::*::*Group
    Match(String),
    /// Exact literal match: "AWS::EC2::Instance"
    Exact(String),
    /// Match starting with the given string: "AWS::EC2::Inst
    StartsWith(String),
}

impl FromStr for SearchQuery {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        parser::parse(s)
    }
}

impl SearchTerm {
    /// Get the inner string, regardless of the type of match
    pub fn as_str(&self) -> &str {
        match self {
            SearchTerm::Exact(s) | SearchTerm::StartsWith(s) | SearchTerm::Match(s) => s,
        }
    }

    /// Match a query term against a JSON value (with case-insensitive string matching)
    pub fn match_value(&self, value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::String(value) => self.match_str(value),
            serde_json::Value::Bool(value) => self.match_bool(*value),
            serde_json::Value::Number(value) => self.match_number(value),
            serde_json::Value::Null => self.match_null(),
            serde_json::Value::Object(_) | serde_json::Value::Array(_) => false,
        }
    }

    /// Match a query term like Instance or "Instance" against a string, case insensitively
    pub fn match_str(&self, value: &str) -> bool {
        // TODO handle international case comparison as well. Library? icu? case_sensitive_string?
        match self {
            SearchTerm::Match(term) => {
                // If we have a*b*c, find a, then b, then c ...
                let mut value = value;
                for part in term.split('*') {
                    let Some((_, remaining)) = value.split_once_ignore_ascii_case(part) else {
                        return false;
                    };
                    value = remaining;
                }
                true
            }
            SearchTerm::Exact(term) => value.eq_ignore_ascii_case(term),
            SearchTerm::StartsWith(term) => value.starts_with_ignore_ascii_case(term),
        }
    }

    /// Match a query term like "1" or "3.14" against a number (e.g. "1.0" matches 1)
    pub fn match_number(&self, value: &serde_json::Number) -> bool {
        // We don't support partial matches for numbers, so we treat quotes and non-quotes the same.
        let term = self.as_str();

        // Try to parse the term as a number
        // TODO perf: don't re-parse the term on every attribute we check against. Float parsing
        // is actually really expensive.
        // TODO(jkeiser) serde_json doesn't support comparing
        term.parse().is_ok_and(|term| value == &term)
    }

    /// Match a query term like "true" or "false" against a bool
    pub fn match_bool(&self, value: bool) -> bool {
        // For bools, we only want to check true or false and don't care whether it's quoted or not.
        let term = self.as_str();

        // TODO t, tr, tru, f, fa, fal should also match given the incremental typing rule
        if term.eq_ignore_ascii_case("true") {
            value
        } else if term.eq_ignore_ascii_case("false") {
            !value
        } else {
            false
        }
    }

    /// Match a query term like "null" against a bool
    pub fn match_null(&self) -> bool {
        // For null, we only want to check null and don't care whether it's quoted or not.
        let term = self.as_str();

        // TODO n, nu, nul should also match given the incremental typing rule
        term.eq_ignore_ascii_case("null")
    }

    /// Match a query term like "01F8MECHZX3TBDSZ7XRADM79XE" against a ULID
    pub fn match_ulid(&self, value: impl Into<Ulid>) -> bool {
        let value = value.into();
        match self {
            SearchTerm::Exact(term) | SearchTerm::Match(term) | SearchTerm::StartsWith(term) => {
                term.parse().is_ok_and(|u| value == u)
            }
        }
    }
}

/// String methods that ignore ascii case (like eq_ignore_ascii_case for other pattern matches)
trait StrIgnoreAsciiCaseHelpers {
    /// Like find(), but ignores ascii case
    fn find_ignore_ascii_case(&self, pattern: &str) -> Option<usize>;

    /// Like split_once(), but ignores ascii case
    fn split_once_ignore_ascii_case<'a>(&'a self, pattern: &str) -> Option<(&'a str, &'a str)>;

    /// Like starts_with(), but ignores ascii case
    fn starts_with_ignore_ascii_case(&self, prefix: &str) -> bool;
}

impl StrIgnoreAsciiCaseHelpers for str {
    fn find_ignore_ascii_case(&self, pattern: &str) -> Option<usize> {
        let pattern = pattern.as_bytes();
        let bytes = self.as_bytes();
        if pattern.len() > self.len() {
            return None;
        }

        (0..=(bytes.len() - pattern.len()))
            .find(|&i| bytes[i..i + pattern.len()].eq_ignore_ascii_case(pattern))
    }

    fn split_once_ignore_ascii_case<'a>(&'a self, pattern: &str) -> Option<(&'a str, &'a str)> {
        self.find_ignore_ascii_case(pattern)
            .map(|i| (&self[..i], &self[i + pattern.len()..]))
    }

    fn starts_with_ignore_ascii_case(&self, prefix: &str) -> bool {
        self.get(..prefix.len())
            .is_some_and(|slice| slice.eq_ignore_ascii_case(prefix))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use serde_json::Number;

    use super::*;

    #[test]
    fn match_str() {
        let term = SearchTerm::Match("Instance".to_string());
        assert!(term.match_str("Instance")); // exact
        assert!(term.match_str("instance")); // case
        assert!(term.match_str("Instances")); // suffix
        assert!(term.match_str("AWS::EC2::Instance")); // prefix
        assert!(term.match_str("AWS::EC2::Instances")); // prefix+suffix
        assert!(!term.match_str("Inst")); // partial (start)
        assert!(!term.match_str("stan")); // partial (middle)
        assert!(!term.match_str("ance")); // partial (end)
        assert!(!term.match_str("")); // empty
        assert!(!term.match_str("foo")); // something else entirely
        assert!(!term.match_str("foobario")); // something else entirely of the same length

        let value = "Instance";
        assert!(SearchTerm::Match("*Stan*".to_string()).match_str(value));
        assert!(SearchTerm::Match("*St*an*".to_string()).match_str(value));
        assert!(SearchTerm::Match("*S*n*".to_string()).match_str(value));
        assert!(SearchTerm::Match("n*t*n*e".to_string()).match_str(value));
        assert!(SearchTerm::Match("**".to_string()).match_str(value));
        assert!(SearchTerm::Match("n*".to_string()).match_str(value));
        assert!(SearchTerm::Match("**ta**c**e**".to_string()).match_str(value));
        assert!(!SearchTerm::Match("n*t*x".to_string()).match_str(value));
        assert!(!SearchTerm::Match("*an*St*".to_string()).match_str(value));
        assert!(!SearchTerm::Match("*an*St*".to_string()).match_str(value));
        assert!(!SearchTerm::Match("*a*cee*".to_string()).match_str(value));
    }

    #[test]
    fn match_str_exact() {
        let term = SearchTerm::Exact("Instance".to_string());
        assert!(term.match_str("Instance")); // exact
        assert!(term.match_str("instance")); // case
        assert!(!term.match_str("Instances")); // suffix
        assert!(!term.match_str("AWS::EC2::Instance")); // prefix
        assert!(!term.match_str("AWS::EC2::Instances")); // prefix+suffix
        assert!(!term.match_str("Inst")); // partial (start)
        assert!(!term.match_str("stan")); // partial (middle)
        assert!(!term.match_str("ance")); // partial (end)
        assert!(!term.match_str("")); // empty
        assert!(!term.match_str("foo")); // something else entirely
        assert!(!term.match_str("foobario")); // something else entirely of the same length
    }

    #[test]
    fn match_str_starts_with() {
        let term = SearchTerm::StartsWith("Instance".to_string());
        assert!(term.match_str("Instance")); // exact
        assert!(term.match_str("instance")); // case
        assert!(term.match_str("Instances")); // suffix
        assert!(!term.match_str("AWS::EC2::Instance")); // prefix
        assert!(!term.match_str("AWS::EC2::Instances")); // prefix+suffix
        assert!(!term.match_str("Inst")); // partial (start)
        assert!(!term.match_str("stan")); // partial (middle)
        assert!(!term.match_str("ance")); // partial (end)
        assert!(!term.match_str("")); // empty
        assert!(!term.match_str("foo")); // something else entirely
        assert!(!term.match_str("foobario")); // something else entirely of the same length
    }

    #[test]
    fn match_null() {
        assert!(SearchTerm::Match("null".to_string()).match_null());
        assert!(SearchTerm::Match("NULL".to_string()).match_null());
        // TODO These partial ones should eventually match!
        assert!(!SearchTerm::Match("nul".to_string()).match_null());
        assert!(!SearchTerm::Match("nu".to_string()).match_null());
        assert!(!SearchTerm::Match("n".to_string()).match_null());

        assert!(!SearchTerm::Match("nulll".to_string()).match_null());
        assert!(!SearchTerm::Match("nnull".to_string()).match_null());
        assert!(!SearchTerm::Match("true".to_string()).match_null());
        assert!(!SearchTerm::Match("false".to_string()).match_null());
        assert!(!SearchTerm::Match("0".to_string()).match_null());
        assert!(!SearchTerm::Match("1".to_string()).match_null());
    }

    #[test]
    fn match_bool() {
        assert!(SearchTerm::Match("true".to_string()).match_bool(true));
        assert!(SearchTerm::Match("TRUE".to_string()).match_bool(true));
        // TODO These partial ones should eventually match!
        assert!(!SearchTerm::Match("tru".to_string()).match_bool(true));
        assert!(!SearchTerm::Match("tr".to_string()).match_bool(true));
        assert!(!SearchTerm::Match("t".to_string()).match_bool(true));
        assert!(!SearchTerm::Match("truee".to_string()).match_bool(true));
        assert!(!SearchTerm::Match("ttrue".to_string()).match_bool(true));
        assert!(!SearchTerm::Match("true".to_string()).match_bool(false));

        assert!(SearchTerm::Match("false".to_string()).match_bool(false));
        assert!(SearchTerm::Match("FALSE".to_string()).match_bool(false));
        // TODO These partial ones should eventually match!
        assert!(!SearchTerm::Match("fals".to_string()).match_bool(false));
        assert!(!SearchTerm::Match("fal".to_string()).match_bool(false));
        assert!(!SearchTerm::Match("fa".to_string()).match_bool(false));
        assert!(!SearchTerm::Match("f".to_string()).match_bool(false));
        assert!(!SearchTerm::Match("falsee".to_string()).match_bool(false));
        assert!(!SearchTerm::Match("ffalse".to_string()).match_bool(false));
        assert!(!SearchTerm::Match("false".to_string()).match_bool(true));

        assert!(!SearchTerm::Match("null".to_string()).match_bool(true));
        assert!(!SearchTerm::Match("null".to_string()).match_bool(false));
        assert!(!SearchTerm::Match("0".to_string()).match_bool(true));
        assert!(!SearchTerm::Match("0".to_string()).match_bool(false));
        assert!(!SearchTerm::Match("1".to_string()).match_bool(true));
        assert!(!SearchTerm::Match("1".to_string()).match_bool(false));
    }

    #[test]
    fn match_number() {
        assert!(SearchTerm::Match("0".to_string()).match_number(&0.into()));
        // assert!(SearchTerm::Match("-0".to_string()).match_number(&0.into()));

        assert!(SearchTerm::Match("1".to_string()).match_number(&1.into()));
        // assert!(SearchTerm::Match("1.0".to_string()).match_number(&1.into()));
        // assert!(SearchTerm::Match("1e0".to_string()).match_number(&1.into()));
        // assert!(SearchTerm::Match("10e-1".to_string()).match_number(&1.into()));
        // assert!(SearchTerm::Match("0.1e1".to_string()).match_number(&1.into()));
        // assert!(SearchTerm::Match("01".to_string()).match_number(&1.into()));
        // assert!(SearchTerm::Match("01.0e0".to_string()).match_number(&1.into()));
        assert!(
            !SearchTerm::Match("1".to_string()).match_number(&Number::from_f64(1.0001).unwrap())
        );
        assert!(!SearchTerm::Match("1".to_string()).match_number(&0.into()));
        assert!(!SearchTerm::Match("1".to_string()).match_number(&(-1).into()));
        assert!(!SearchTerm::Match("1".to_string()).match_number(&10.into()));

        // assert!(SearchTerm::Match("1".to_string()).match_number(&Number::from_f64(1.0).unwrap()));
        assert!(SearchTerm::Match("1.0".to_string()).match_number(&Number::from_f64(1.0).unwrap()));
        assert!(SearchTerm::Match("1e0".to_string()).match_number(&Number::from_f64(1.0).unwrap()));
        assert!(
            SearchTerm::Match("10e-1".to_string()).match_number(&Number::from_f64(1.0).unwrap())
        );
        assert!(
            SearchTerm::Match("0.1e1".to_string()).match_number(&Number::from_f64(1.0).unwrap())
        );
        // assert!(SearchTerm::Match("01".to_string()).match_number(&Number::from_f64(1.0).unwrap()));
        // assert!(SearchTerm::Match("01.0e0".to_string()).match_number(&1.into()));
        assert!(
            !SearchTerm::Match("1".to_string()).match_number(&Number::from_f64(1.0001).unwrap())
        );
        assert!(!SearchTerm::Match("1".to_string()).match_number(&0.into()));
        assert!(!SearchTerm::Match("1".to_string()).match_number(&(-1).into()));
        assert!(!SearchTerm::Match("1".to_string()).match_number(&10.into()));

        assert!(SearchTerm::Match("-1".to_string()).match_number(&(-1).into()));
        // assert!(SearchTerm::Match("-1.0".to_string()).match_number(&(-1).into()));
        assert!(!SearchTerm::Match("-1".to_string()).match_number(&0.into()));
        assert!(!SearchTerm::Match("-1".to_string()).match_number(&1.into()));

        assert!(SearchTerm::Match("123".to_string()).match_number(&123.into()));
        assert!(!SearchTerm::Match("123".to_string()).match_number(&1.into()));
        assert!(!SearchTerm::Match("123".to_string()).match_number(&124.into()));

        assert!(SearchTerm::Match("1.2".to_string()).match_number(&Number::from_f64(1.2).unwrap()));
        assert!(
            SearchTerm::Match("1.20".to_string()).match_number(&Number::from_f64(1.2).unwrap())
        );
        assert!(
            SearchTerm::Match("12e-1".to_string()).match_number(&Number::from_f64(1.2).unwrap())
        );
        assert!(!SearchTerm::Match("1.2".to_string()).match_number(&1.into()));
        assert!(
            !SearchTerm::Match("1.2".to_string()).match_number(&Number::from_f64(-1.2).unwrap())
        );
        assert!(
            !SearchTerm::Match("1.2".to_string()).match_number(&Number::from_f64(1.2001).unwrap())
        );
        assert!(!SearchTerm::Match("1.2".to_string()).match_number(&1.into()));
    }

    #[test]
    fn find_ignore_ascii_case() {
        assert_eq!("Hello, world".find_ignore_ascii_case("hello"), Some(0));
        assert_eq!("Hello, world".find_ignore_ascii_case("WORLD"), Some(7));
        assert_eq!("Hello, world".find_ignore_ascii_case("o, W"), Some(4));
        assert_eq!("Hello, world".find_ignore_ascii_case("foo"), None);
        assert_eq!("Hello, world".find_ignore_ascii_case(""), Some(0));
        assert_eq!(
            "Hello, world".find_ignore_ascii_case("Hello, world"),
            Some(0)
        );
        assert_eq!("Hello, world".find_ignore_ascii_case("Hello, worldd"), None);
        assert_eq!("".find_ignore_ascii_case("foo"), None);
        assert_eq!("".find_ignore_ascii_case(""), Some(0));
    }

    #[test]
    fn split_once_ignore_ascii_case() {
        assert_eq!(
            "abc DEF def".split_once_ignore_ascii_case("def"),
            Some(("abc ", " def"))
        );
        assert_eq!(
            "abc DEF def".split_once_ignore_ascii_case("def"),
            Some(("abc ", " def"))
        );
        assert_eq!(
            "abc DEF def".split_once_ignore_ascii_case("abc DEF def"),
            Some(("", ""))
        );
        assert_eq!(
            "abc DEF def".split_once_ignore_ascii_case("abc DEF deff"),
            None
        );
        assert_eq!("abc DEF def".split_once_ignore_ascii_case("ghi"), None);
        assert_eq!(
            "abc DEF def".split_once_ignore_ascii_case(""),
            Some(("", "abc DEF def"))
        );
        assert_eq!(
            "abc DEF def".split_once_ignore_ascii_case("c d"),
            Some(("ab", "EF def"))
        );
        assert_eq!("".split_once_ignore_ascii_case("foo"), None);
        assert_eq!("".split_once_ignore_ascii_case(""), Some(("", "")));
    }

    #[test]
    fn starts_with_ignore_ascii_case() {
        assert!("Hello, world".starts_with_ignore_ascii_case("hello"));
        assert!("Hello, world".starts_with_ignore_ascii_case("HELLO"));
        assert!(!"Hello, world".starts_with_ignore_ascii_case("world"));
        assert!(!"Hello, world".starts_with_ignore_ascii_case("foo"));
        assert!("Hello, world".starts_with_ignore_ascii_case(""));
        assert!(!"".starts_with_ignore_ascii_case("foo"));
        assert!("".starts_with_ignore_ascii_case(""));
    }
}
