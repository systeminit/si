use std::{
    fmt,
    fmt::{
        Display,
        Formatter,
    },
};

use regex::Regex;
use serde::{
    Deserialize,
    Serialize,
    Serializer,
};
use si_frontend_types::ConnectionAnnotation as FeConnectionAnnotation;
use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ConnectionAnnotationError {
    #[error("badly formed connection annotation: {0}")]
    BadFormat(String),
    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),
}

#[derive(Clone, Eq, Debug, PartialEq, Deserialize, Serialize)]
pub struct ConnectionAnnotation {
    pub tokens: Vec<String>,
}

impl From<ConnectionAnnotation> for FeConnectionAnnotation {
    fn from(value: ConnectionAnnotation) -> Self {
        Self {
            tokens: value.tokens.clone(),
        }
    }
}

impl TryFrom<String> for ConnectionAnnotation {
    type Error = ConnectionAnnotationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // A connection annotation is composed by a series of pairs, with the following recursive
        // structure:
        // PAIR ::= TOKEN<PAIR> | TOKEN
        // where TOKEN is any combination of word characters (\w regex matcher) and single spaces,
        // and < and > are those literal characters
        let mut tokens = vec![];

        let re = Regex::new(r"^(?<token>[\w ]+)(?:<(?<tail>.+)>)?$")?;

        let mut this_value = value;
        loop {
            let captures = re
                .captures(&this_value)
                .ok_or(Self::Error::BadFormat(this_value.clone()))?;
            let token = captures
                .name("token")
                .ok_or(Self::Error::BadFormat(this_value.clone()))?
                .as_str();
            tokens.push(token.to_string());

            let maybe_tail = captures.name("tail");
            if let Some(tail) = maybe_tail {
                this_value = tail.as_str().to_string();
            } else {
                break;
            }
        }

        Ok(ConnectionAnnotation { tokens })
    }
}

impl ConnectionAnnotation {
    pub fn target_fits_reference(target_ca: &Self, reference_ca: &Self) -> bool {
        let annotation_src = &target_ca.tokens;
        let annotation_dest = &reference_ca.tokens;

        if annotation_dest.len() > annotation_src.len() {
            return false;
        }

        let annotation_dest_last_index = annotation_dest.len() - 1;
        let annotation_src_last_index = annotation_src.len() - 1;

        for i in 0..annotation_dest.len() {
            if annotation_dest[annotation_dest_last_index - i].to_lowercase()
                != annotation_src[annotation_src_last_index - i].to_lowercase()
            {
                return false;
            }
        }

        true
    }

    pub fn from_tokens_array(tokens: Vec<String>) -> Self {
        Self { tokens }
    }
}

impl Display for ConnectionAnnotation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut out = self.tokens.last().ok_or(fmt::Error)?.clone();
        for token in self.tokens.iter().rev().skip(1) {
            out = format!("{token}<{out}>");
        }

        f.serialize_str(out.as_str())
    }
}

#[test]
fn deserialize_connection_annotation() {
    let cases = vec![
        ("arn", vec!["arn"]),
        ("arn<string>", vec!["arn", "string"]),
        ("user_arn<arn<string>>", vec!["user_arn", "arn", "string"]),
    ];

    for (raw, tokens) in cases {
        let ca =
            ConnectionAnnotation::try_from(raw.to_string()).expect("parse connection annotation");
        assert_eq!(ca.tokens, tokens)
    }
}

#[test]
fn serialize_connection_annotation() {
    let cases = vec![
        (vec!["arn"], "arn"),
        (vec!["arn", "string"], "arn<string>"),
        (vec!["user_arn", "arn", "string"], "user_arn<arn<string>>"),
    ];

    for (tokens, raw) in cases {
        let ca = ConnectionAnnotation {
            tokens: tokens.iter().map(ToString::to_string).collect(),
        };
        assert_eq!(ca.to_string(), raw)
    }
}

#[test]
fn connection_annotation_fits() {
    let cases_and_results = vec![
        ("arn", "arn", true),
        ("arn<string>", "arn<string>", true),
        ("user_arn<arn<string>>", "user_arn<arn<string>>", true),
        ("arn<string>", "string", true),
        ("string", "arn<string>", false),
        ("User Data", "user data", true), // fix edge connections being different cases
    ];

    for (raw_target, raw_reference, result) in cases_and_results {
        let target = ConnectionAnnotation::try_from(raw_target.to_string())
            .expect("parse object annotation");
        let reference = ConnectionAnnotation::try_from(raw_reference.to_string())
            .expect("parse slot annotation");

        assert_eq!(
            ConnectionAnnotation::target_fits_reference(&target, &reference),
            result
        )
    }
}
