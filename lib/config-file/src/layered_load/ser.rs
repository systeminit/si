use std::{
    collections::HashMap,
    fmt::{
        Debug,
        Display,
    },
    num::TryFromIntError,
};

use config::Value;
use serde::{
    Serialize,
    ser::{
        self,
        Impossible,
    },
};
use thiserror::Error;

pub fn to_hash_map<T>(value: &T) -> Result<HashMap<String, Value>>
where
    T: Serialize,
{
    let mut serializer = Serializer::default();
    value.serialize(&mut serializer)?;
    Ok(serializer.into_inner())
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SerializerError {
    #[error("cannot convert integer--will under/overflow")]
    Int(#[from] TryFromIntError),
    #[error("current key is empty, cannot insert into map")]
    KeyEmpty,
    #[error("key must be a string")]
    KeyMustBeAString,
    #[error("{0}")]
    Message(String),
    #[error("current array is empty, cannot insert into array")]
    NoArray,
}

type Result<T> = std::result::Result<T, SerializerError>;

impl ser::Error for SerializerError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        SerializerError::Message(msg.to_string())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Serializer {
    output: HashMap<String, Value>,
    key: String,
    array: Option<Vec<Value>>,
}

impl Serializer {
    pub fn into_inner(self) -> HashMap<String, Value> {
        self.output
    }

    fn insert_boolean(&mut self, key: String, value: impl Into<bool>) -> Result<()> {
        self.insert(key, Value::new(None, value.into()))
    }

    fn insert_i64(&mut self, key: String, value: impl Into<i64>) -> Result<()> {
        self.insert(key, Value::new(None, value.into()))
    }

    fn insert_f64(&mut self, key: String, value: impl Into<f64>) -> Result<()> {
        self.insert(key, Value::new(None, value.into()))
    }

    fn insert_str(&mut self, key: String, value: &str) -> Result<()> {
        self.insert(key, Value::new(None, value))
    }

    fn insert(&mut self, key: String, value: Value) -> Result<()> {
        match &mut self.array {
            Some(array) => {
                array.push(value);
            }
            None => {
                if self.key.is_empty() {
                    return Err(SerializerError::KeyEmpty);
                }
                self.output.insert(key, value);
            }
        }
        Ok(())
    }
}

impl ser::Serializer for &mut Serializer {
    type Ok = ();
    type Error = SerializerError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok> {
        self.insert_boolean(self.key.clone(), value)
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        self.insert_i64(self.key.clone(), value)
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok> {
        self.insert_i64(self.key.clone(), value)
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok> {
        self.insert_i64(self.key.clone(), value)
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok> {
        self.insert_i64(self.key.clone(), value)
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        self.insert_i64(self.key.clone(), value)
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok> {
        self.insert_i64(self.key.clone(), value)
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        self.insert_i64(self.key.clone(), value)
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        self.insert_i64(self.key.clone(), i64::try_from(value)?)
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok> {
        self.insert_f64(self.key.clone(), value)
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok> {
        self.insert_f64(self.key.clone(), value)
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        // A char encoded as UTF-8 takes 4 bytes at most
        let mut buf = [0; 4];
        self.insert_str(self.key.clone(), value.encode_utf8(&mut buf))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        self.insert_str(self.key.clone(), value)
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(value.len()))?;
        for byte in value {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.insert(self.key.clone(), Value::new(None, None::<i64>))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.array = Some(match len {
            Some(len) => Vec::with_capacity(len),
            None => Vec::new(),
        });

        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        todo!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        // Append variant name to current key path, just like entering a nested struct
        if !self.key.is_empty() {
            self.key.push('.');
        }
        self.key.push_str(variant);
        Ok(self)
    }
}

impl ser::SerializeSeq for &mut Serializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        match self.array.take() {
            Some(array) => {
                self.output
                    .insert(self.key.clone(), Value::new(None, array));
                self.array = None;
            }
            None => return Err(SerializerError::NoArray),
        }
        Ok(())
    }
}

impl ser::SerializeTuple for &mut Serializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for &mut Serializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleVariant for &mut Serializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}

impl ser::SerializeMap for &mut Serializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        key.serialize(MapKeySerializer { ser: self })?;
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut **self)?;
        match self.key.rfind('.') {
            Some(at) => {
                let _ = self.key.split_off(at);
            }
            _ => self.key.clear(),
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl ser::SerializeStruct for &mut Serializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        if !self.key.is_empty() {
            self.key.push('.');
        }
        self.key.push_str(key);
        value.serialize(&mut **self)?;
        match self.key.rfind('.') {
            Some(at) => {
                let _ = self.key.split_off(at);
            }
            _ => self.key.clear(),
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut Serializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        if !self.key.is_empty() {
            self.key.push('.');
        }
        self.key.push_str(key);
        value.serialize(&mut **self)?;
        match self.key.rfind('.') {
            Some(at) => {
                let _ = self.key.split_off(at);
            }
            _ => self.key.clear(),
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        // Pop the variant name we added in serialize_struct_variant
        // This restores the key to its state before the variant
        match self.key.rfind('.') {
            Some(at) => {
                let _ = self.key.split_off(at);
            }
            _ => self.key.clear(),
        }
        Ok(())
    }
}

fn key_must_be_a_string() -> SerializerError {
    SerializerError::KeyMustBeAString
}

struct MapKeySerializer<'a> {
    ser: &'a mut Serializer,
}

impl MapKeySerializer<'_> {
    fn push_key(&mut self, key: &str) {
        if !self.ser.key.is_empty() {
            self.ser.key.push('.');
        }
        self.ser.key.push_str(key);
    }
}

impl ser::Serializer for MapKeySerializer<'_> {
    type Ok = ();
    type Error = SerializerError;

    type SerializeSeq = Impossible<(), SerializerError>;
    type SerializeTuple = Impossible<(), SerializerError>;
    type SerializeTupleStruct = Impossible<(), SerializerError>;
    type SerializeTupleVariant = Impossible<(), SerializerError>;
    type SerializeMap = Impossible<(), SerializerError>;
    type SerializeStruct = Impossible<(), SerializerError>;
    type SerializeStructVariant = Impossible<(), SerializerError>;

    fn serialize_bool(self, _value: bool) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_i8(mut self, value: i8) -> Result<()> {
        self.push_key(&value.to_string());
        Ok(())
    }

    fn serialize_i16(mut self, value: i16) -> Result<()> {
        self.push_key(&value.to_string());
        Ok(())
    }

    fn serialize_i32(mut self, value: i32) -> Result<()> {
        self.push_key(&value.to_string());
        Ok(())
    }

    fn serialize_i64(mut self, value: i64) -> Result<()> {
        self.push_key(&value.to_string());
        Ok(())
    }

    fn serialize_u8(mut self, value: u8) -> Result<()> {
        self.push_key(&value.to_string());
        Ok(())
    }

    fn serialize_u16(mut self, value: u16) -> Result<()> {
        self.push_key(&value.to_string());
        Ok(())
    }

    fn serialize_u32(mut self, value: u32) -> Result<()> {
        self.push_key(&value.to_string());
        Ok(())
    }

    fn serialize_u64(mut self, value: u64) -> Result<()> {
        self.push_key(&value.to_string());
        Ok(())
    }

    fn serialize_f32(mut self, value: f32) -> Result<()> {
        self.push_key(&value.to_string());
        Ok(())
    }

    fn serialize_f64(mut self, value: f64) -> Result<()> {
        self.push_key(&value.to_string());
        Ok(())
    }

    fn serialize_char(mut self, value: char) -> Result<()> {
        self.push_key(&value.to_string());
        Ok(())
    }

    fn serialize_str(mut self, value: &str) -> Result<()> {
        self.push_key(value);
        Ok(())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(key_must_be_a_string())
    }
}
