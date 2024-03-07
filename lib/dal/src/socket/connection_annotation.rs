use regex::Regex;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use std::fmt::{Display, Formatter};
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
    tokens: Vec<String>,
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

        let mut this_value = value;
        loop {
            let re = Regex::new(r"^(?<token>[\w ]+)(?:<(?<tail>.+)>)?$")?;

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
    pub fn target_fits_reference(target_ca: Self, reference_ca: Self) -> bool {
        let annotation_src = target_ca.tokens;
        let annotation_dest = reference_ca.tokens;

        annotation_src.len() >= annotation_dest.len()
            && annotation_src.as_slice()[annotation_src.len() - annotation_dest.len()..].to_vec()
                == annotation_dest
    }
}

impl Display for ConnectionAnnotation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.tokens.len() < 1 {
            return Err(fmt::Error);
        }
        let mut out = self.tokens[0].clone();
        for token in self.tokens.iter().rev().skip(1) {
            out = format!("{}<{}>", token, out);
        }

        f.serialize_str(out.as_str())
    }
}

#[test]
fn deserialize_connection_annotation() {
    let cases = vec![
        ("arn", vec!["arn"]),
        ("arn<string>", vec!["arn", "string"]),
        ("userArn<arn<string>>", vec!["userArn", "arn", "string"]),
    ];

    for (raw, tokens) in cases {
        let ca =
            ConnectionAnnotation::try_from(raw.to_string()).expect("parse connection annotation");
        assert_eq!(ca.tokens, tokens)
    }
}
