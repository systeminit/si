use std::collections::BTreeMap;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, Clone)]
pub enum CasValueNumber {
    U64(u64),
    I64(i64),
    F64(f64),
}

// (Comment copied from serde_json::Number's equivalent "N" type)
// Implementing Eq is fine since any float values are always finite.
impl Eq for CasValueNumber {}

/// A type that can be converted to and from serde_json::Value types infallibly,
/// *so long as* arbitrary precision arithmetic is not enabled for serde_json.
/// This is necessary because postcard will *not* deserialize serde_json's `Number`
/// type, but we still want to store arbitrary payloads in our content store.
/// The alternative is to serialize the value to a string and then serialize
/// that string with postcard.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, Clone)]
#[remain::sorted]
pub enum CasValue {
    /// An array of values
    Array(Vec<CasValue>),
    /// A boolean scalar
    Bool(bool),
    /// A null value
    Null,
    /// A Number value. JSON numbers are either double precision IEEE floating point values, or
    /// they in some implementations can be BigInt values. However, we're currently only going to
    /// support double precision floats and 64 bit integers. If arbitrary precision integers are
    /// enabled for serde_json, this *will* cause a panic.
    Number(CasValueNumber),
    /// An object. BTreeMap is the internal representation used by serde_json for objects,
    /// *unless* order preservation is enabled. If order preservation is enabled, we will
    /// lose that ordering information in the conversion to/from `serde_json::Value``.
    Object(BTreeMap<String, CasValue>),
    /// A string scalar value
    String(String),
}

// todo: make this non-recursive for maps and arrays
impl From<serde_json::Value> for CasValue {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Bool(b),
            serde_json::Value::Number(n) => CasValue::Number(if n.is_u64() {
                CasValueNumber::U64(
                    n.as_u64()
                        .expect("serde_json said it was a u64 but refused to give me one"),
                )
            } else if n.is_i64() {
                CasValueNumber::I64(
                    n.as_i64()
                        .expect("serde_json said it was an i64 but refused to give me one"),
                )
            } else if n.is_f64() {
                CasValueNumber::F64(
                    n.as_f64()
                        .expect("serde_json said it was an f64 but refused to give me one"),
                )
            } else {
                panic!("the arbitrary_precision feature of serde_json is not supported");
            }),
            serde_json::Value::Array(mut a) => Self::Array(a.drain(..).map(|e| e.into()).collect()),
            serde_json::Value::String(s) => Self::String(s),
            // Can we avoid these clones?
            serde_json::Value::Object(map) => Self::Object(
                map.iter()
                    .map(|(k, v)| (k.to_owned(), v.to_owned().into()))
                    .collect(),
            ),
        }
    }
}

impl From<CasValue> for serde_json::Value {
    fn from(value: CasValue) -> Self {
        match value {
            CasValue::Null => serde_json::Value::Null,
            CasValue::Bool(b) => serde_json::Value::Bool(b),
            CasValue::Array(mut a) => {
                serde_json::Value::Array(a.drain(..).map(Into::into).collect())
            }
            CasValue::Number(n) => serde_json::Value::Number(match n {
                CasValueNumber::U64(n) => n.into(),
                CasValueNumber::I64(n) => n.into(),
                CasValueNumber::F64(n) => serde_json::value::Number::from_f64(n)
                    .expect("cannot deserialize an infinite or NAN f64 value"),
            }),
            CasValue::String(s) => serde_json::Value::String(s),
            CasValue::Object(map) => serde_json::Value::Object(
                map.iter()
                    .map(|(k, v)| (k.to_owned(), v.to_owned().into()))
                    .collect(),
            ),
        }
    }
}
