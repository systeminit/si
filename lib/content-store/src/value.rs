use std::collections::BTreeMap;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, Clone)]
pub enum ValueNumber {
    U64(u64),
    I64(i64),
    F64(f64),
}

/// A type that can be converted to and from serde_json::Value types infallibly,
/// *so long as* arbitrary precision arithmetic is not enabled for serde_json.
/// This is necessary because postcard will *not* deserialize serde_json's `Number`
/// type, but we still want to store arbitrary payloads in our content store.
/// The alternative is to serialize the value to a string and then serialize
/// that string with postcard.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, Clone)]
#[remain::sorted]
pub enum Value {
    /// An array of values
    Array(Vec<Value>),
    /// A boolean scalar
    Bool(bool),
    /// A null value
    Null,
    /// A Number value. JSON numbers are either double precision IEEE floating point values, or
    /// they in some implementations can be BigInt values. However, we're currently only going to
    /// support double precision floats and 64 bit integers. If arbitrary precision integers are
    /// enabled for serde_json, this *will* cause a panic.
    Number(ValueNumber),
    /// An object. BTreeMap is the internal representation used by serde_json for objects,
    /// *unless* order preservation is enabled. If order preservation is enabled, we will
    /// lose that ordering information in the conversion to/from `serde_json::Value``.
    Object(BTreeMap<String, Value>),
    /// A string scalar value
    String(String),
}

// todo: make this non-recursive for maps and arrays
impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Bool(b),
            serde_json::Value::Number(n) => Value::Number(if n.is_u64() {
                ValueNumber::U64(
                    n.as_u64()
                        .expect("serde_json said it was a u64 but refused to give me one"),
                )
            } else if n.is_i64() {
                ValueNumber::I64(
                    n.as_i64()
                        .expect("serde_json said it was an i64 but refused to give me one"),
                )
            } else if n.is_f64() {
                ValueNumber::F64(
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

impl From<Value> for serde_json::Value {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => serde_json::Value::Null,
            Value::Bool(b) => serde_json::Value::Bool(b),
            Value::Array(mut a) => serde_json::Value::Array(a.drain(..).map(Into::into).collect()),
            Value::Number(n) => serde_json::Value::Number(match n {
                ValueNumber::U64(n) => n.into(),
                ValueNumber::I64(n) => n.into(),
                ValueNumber::F64(n) => serde_json::value::Number::from_f64(n)
                    .expect("cannot deserialize an infinite or NAN f64 value"),
            }),
            Value::String(s) => serde_json::Value::String(s),
            Value::Object(map) => serde_json::Value::Object(
                map.iter()
                    .map(|(k, v)| (k.to_owned(), v.to_owned().into()))
                    .collect(),
            ),
        }
    }
}
