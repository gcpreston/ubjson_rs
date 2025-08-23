//! Serde integration for UBJSON serialization and deserialization.
//!
//! This module provides implementations of serde's Serializer and Deserializer traits
//! to enable automatic serialization of Rust types using derive macros.

#[cfg(feature = "serde")]
use serde::{ser, de};
use std::io::{Write, Read};
use crate::{UbjsonSerializer, UbjsonDeserializer, UbjsonError, UbjsonValue};

#[cfg(feature = "serde")]
impl<W: Write> ser::Serializer for UbjsonSerializer<W> {
    type Ok = ();
    type Error = UbjsonError;
    type SerializeSeq = SerializeSeq<W>;
    type SerializeTuple = SerializeSeq<W>;
    type SerializeTupleStruct = SerializeSeq<W>;
    type SerializeTupleVariant = SerializeTupleVariant<W>;
    type SerializeMap = SerializeMap<W>;
    type SerializeStruct = SerializeMap<W>;
    type SerializeStructVariant = SerializeStructVariant<W>;

    fn serialize_bool(mut self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.serialize_value(&UbjsonValue::Bool(v))
    }

    fn serialize_i8(mut self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_value(&UbjsonValue::Int8(v))
    }

    fn serialize_i16(mut self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_value(&UbjsonValue::Int16(v))
    }

    fn serialize_i32(mut self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_value(&UbjsonValue::Int32(v))
    }

    fn serialize_i64(mut self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_value(&UbjsonValue::Int64(v))
    }

    fn serialize_u8(mut self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_value(&UbjsonValue::UInt8(v))
    }

    fn serialize_u16(mut self, v: u16) -> Result<Self::Ok, Self::Error> {
        // UBJSON doesn't have u16, use i32 if it fits, otherwise i64
        if v <= i32::MAX as u16 {
            self.serialize_value(&UbjsonValue::Int32(v as i32))
        } else {
            self.serialize_value(&UbjsonValue::Int64(v as i64))
        }
    }

    fn serialize_u32(mut self, v: u32) -> Result<Self::Ok, Self::Error> {
        // UBJSON doesn't have u32, use i64 to ensure it fits
        self.serialize_value(&UbjsonValue::Int64(v as i64))
    }

    fn serialize_u64(mut self, v: u64) -> Result<Self::Ok, Self::Error> {
        // UBJSON doesn't have u64, check if it fits in i64
        if v <= i64::MAX as u64 {
            self.serialize_value(&UbjsonValue::Int64(v as i64))
        } else {
            // Use high-precision number for values that don't fit in i64
            self.serialize_value(&UbjsonValue::HighPrecision(v.to_string()))
        }
    }

    fn serialize_f32(mut self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_value(&UbjsonValue::Float32(v))
    }

    fn serialize_f64(mut self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.serialize_value(&UbjsonValue::Float64(v))
    }

    fn serialize_char(mut self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_value(&UbjsonValue::Char(v))
    }

    fn serialize_str(mut self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize_value(&UbjsonValue::String(v.to_string()))
    }

    fn serialize_bytes(mut self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        // Serialize bytes as an array of uint8 values
        let byte_values: Vec<UbjsonValue> = v.iter()
            .map(|&b| UbjsonValue::UInt8(b))
            .collect();
        self.serialize_value(&UbjsonValue::Array(byte_values))
    }

    fn serialize_none(mut self) -> Result<Self::Ok, Self::Error> {
        self.serialize_value(&UbjsonValue::Null)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(mut self) -> Result<Self::Ok, Self::Error> {
        self.serialize_value(&UbjsonValue::Null)
    }

    fn serialize_unit_struct(mut self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_value(&UbjsonValue::Null)
    }

    fn serialize_unit_variant(
        mut self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_value(&UbjsonValue::String(variant.to_string()))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        mut self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        let ubjson_value = to_ubjson_value(value)?;
        let mut map = std::collections::HashMap::new();
        map.insert(variant.to_string(), ubjson_value);
        self.serialize_value(&UbjsonValue::Object(map))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq {
            serializer: self,
            elements: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeTupleVariant {
            serializer: self,
            variant: variant.to_string(),
            elements: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap {
            serializer: self,
            pairs: std::collections::HashMap::new(),
            current_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeStructVariant {
            serializer: self,
            variant: variant.to_string(),
            pairs: std::collections::HashMap::with_capacity(len),
        })
    }
}

// Helper struct for serializing sequences
#[cfg(feature = "serde")]
pub struct SerializeSeq<W: Write> {
    serializer: UbjsonSerializer<W>,
    elements: Vec<UbjsonValue>,
}

#[cfg(feature = "serde")]
impl<W: Write> ser::SerializeSeq for SerializeSeq<W> {
    type Ok = ();
    type Error = UbjsonError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        // Convert the value to UbjsonValue using a helper
        let ubjson_value = to_ubjson_value(value)?;
        self.elements.push(ubjson_value);
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_value(&UbjsonValue::Array(self.elements))
    }
}

#[cfg(feature = "serde")]
impl<W: Write> ser::SerializeTuple for SerializeSeq<W> {
    type Ok = ();
    type Error = UbjsonError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

#[cfg(feature = "serde")]
impl<W: Write> ser::SerializeTupleStruct for SerializeSeq<W> {
    type Ok = ();
    type Error = UbjsonError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

// Helper struct for serializing tuple variants
#[cfg(feature = "serde")]
pub struct SerializeTupleVariant<W: Write> {
    serializer: UbjsonSerializer<W>,
    variant: String,
    elements: Vec<UbjsonValue>,
}

#[cfg(feature = "serde")]
impl<W: Write> ser::SerializeTupleVariant for SerializeTupleVariant<W> {
    type Ok = ();
    type Error = UbjsonError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let ubjson_value = to_ubjson_value(value)?;
        self.elements.push(ubjson_value);
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        let mut map = std::collections::HashMap::new();
        map.insert(self.variant, UbjsonValue::Array(self.elements));
        self.serializer.serialize_value(&UbjsonValue::Object(map))
    }
}

// Helper struct for serializing maps
#[cfg(feature = "serde")]
pub struct SerializeMap<W: Write> {
    serializer: UbjsonSerializer<W>,
    pairs: std::collections::HashMap<String, UbjsonValue>,
    current_key: Option<String>,
}

#[cfg(feature = "serde")]
impl<W: Write> ser::SerializeMap for SerializeMap<W> {
    type Ok = ();
    type Error = UbjsonError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let ubjson_value = to_ubjson_value(key)?;
        
        // Convert the key to a string
        let key_string = match ubjson_value {
            UbjsonValue::String(s) => s,
            UbjsonValue::Char(c) => c.to_string(),
            UbjsonValue::Int8(n) => n.to_string(),
            UbjsonValue::UInt8(n) => n.to_string(),
            UbjsonValue::Int16(n) => n.to_string(),
            UbjsonValue::Int32(n) => n.to_string(),
            UbjsonValue::Int64(n) => n.to_string(),
            UbjsonValue::Float32(n) => n.to_string(),
            UbjsonValue::Float64(n) => n.to_string(),
            UbjsonValue::Bool(b) => b.to_string(),
            _ => return Err(UbjsonError::serde("Map keys must be convertible to strings")),
        };
        
        self.current_key = Some(key_string);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let key = self.current_key.take()
            .ok_or_else(|| UbjsonError::serde("serialize_value called without serialize_key"))?;
        
        let ubjson_value = to_ubjson_value(value)?;
        self.pairs.insert(key, ubjson_value);
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_value(&UbjsonValue::Object(self.pairs))
    }
}

#[cfg(feature = "serde")]
impl<W: Write> ser::SerializeStruct for SerializeMap<W> {
    type Ok = ();
    type Error = UbjsonError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        ser::SerializeMap::serialize_key(self, key)?;
        ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeMap::end(self)
    }
}

// Helper struct for serializing struct variants
#[cfg(feature = "serde")]
pub struct SerializeStructVariant<W: Write> {
    serializer: UbjsonSerializer<W>,
    variant: String,
    pairs: std::collections::HashMap<String, UbjsonValue>,
}

#[cfg(feature = "serde")]
impl<W: Write> ser::SerializeStructVariant for SerializeStructVariant<W> {
    type Ok = ();
    type Error = UbjsonError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let ubjson_value = to_ubjson_value(value)?;
        self.pairs.insert(key.to_string(), ubjson_value);
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        let mut map = std::collections::HashMap::new();
        map.insert(self.variant, UbjsonValue::Object(self.pairs));
        self.serializer.serialize_value(&UbjsonValue::Object(map))
    }
}

// Helper function to convert any serializable value to UbjsonValue
#[cfg(feature = "serde")]
fn to_ubjson_value<T: ?Sized>(value: &T) -> Result<UbjsonValue, UbjsonError>
where
    T: ser::Serialize,
{
    let mut buffer = Vec::new();
    let serializer = UbjsonSerializer::new(&mut buffer);
    value.serialize(serializer)?;
    
    let mut deserializer = UbjsonDeserializer::new(buffer.as_slice());
    deserializer.deserialize_value()
}

// Deserializer implementation
#[cfg(feature = "serde")]
impl<'de, R: Read> de::Deserializer<'de> for UbjsonDeserializer<R> {
    type Error = UbjsonError;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        
        // Special handling for potential enums: if it's an object with exactly one key-value pair,
        // it might be an enum variant, so try to deserialize it as an enum first
        match value {
            UbjsonValue::Object(mut obj) if obj.len() == 1 => {
                // This could be an enum variant, try enum deserialization
                let (variant, variant_value) = obj.drain().next().unwrap();
                visitor.visit_enum(EnumDeserializer::new(variant, variant_value))
            }
            _ => self.deserialize_ubjson_value(value, visitor)
        }
    }

    fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::Bool(b) => visitor.visit_bool(b),
            _ => Err(UbjsonError::serde(format!("Expected bool, found {}", value.type_name()))),
        }
    }

    fn deserialize_i8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::Int8(n) => visitor.visit_i8(n),
            UbjsonValue::UInt8(n) if n <= i8::MAX as u8 => visitor.visit_i8(n as i8),
            _ => Err(UbjsonError::serde(format!("Expected i8, found {}", value.type_name()))),
        }
    }

    fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::Int8(n) => visitor.visit_i16(n as i16),
            UbjsonValue::UInt8(n) => visitor.visit_i16(n as i16),
            UbjsonValue::Int16(n) => visitor.visit_i16(n),
            _ => Err(UbjsonError::serde(format!("Expected i16, found {}", value.type_name()))),
        }
    }

    fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::Int8(n) => visitor.visit_i32(n as i32),
            UbjsonValue::UInt8(n) => visitor.visit_i32(n as i32),
            UbjsonValue::Int16(n) => visitor.visit_i32(n as i32),
            UbjsonValue::Int32(n) => visitor.visit_i32(n),
            _ => Err(UbjsonError::serde(format!("Expected i32, found {}", value.type_name()))),
        }
    }

    fn deserialize_i64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::Int8(n) => visitor.visit_i64(n as i64),
            UbjsonValue::UInt8(n) => visitor.visit_i64(n as i64),
            UbjsonValue::Int16(n) => visitor.visit_i64(n as i64),
            UbjsonValue::Int32(n) => visitor.visit_i64(n as i64),
            UbjsonValue::Int64(n) => visitor.visit_i64(n),
            _ => Err(UbjsonError::serde(format!("Expected i64, found {}", value.type_name()))),
        }
    }

    fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::UInt8(n) => visitor.visit_u8(n),
            UbjsonValue::Int8(n) if n >= 0 => visitor.visit_u8(n as u8),
            _ => Err(UbjsonError::serde(format!("Expected u8, found {}", value.type_name()))),
        }
    }

    fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::UInt8(n) => visitor.visit_u16(n as u16),
            UbjsonValue::Int8(n) if n >= 0 => visitor.visit_u16(n as u16),
            UbjsonValue::Int16(n) if n >= 0 => visitor.visit_u16(n as u16),
            UbjsonValue::Int32(n) if n >= 0 && n <= u16::MAX as i32 => visitor.visit_u16(n as u16),
            _ => Err(UbjsonError::serde(format!("Expected u16, found {}", value.type_name()))),
        }
    }

    fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::UInt8(n) => visitor.visit_u32(n as u32),
            UbjsonValue::Int8(n) if n >= 0 => visitor.visit_u32(n as u32),
            UbjsonValue::Int16(n) if n >= 0 => visitor.visit_u32(n as u32),
            UbjsonValue::Int32(n) if n >= 0 => visitor.visit_u32(n as u32),
            UbjsonValue::Int64(n) if n >= 0 && n <= u32::MAX as i64 => visitor.visit_u32(n as u32),
            _ => Err(UbjsonError::serde(format!("Expected u32, found {}", value.type_name()))),
        }
    }

    fn deserialize_u64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::UInt8(n) => visitor.visit_u64(n as u64),
            UbjsonValue::Int8(n) if n >= 0 => visitor.visit_u64(n as u64),
            UbjsonValue::Int16(n) if n >= 0 => visitor.visit_u64(n as u64),
            UbjsonValue::Int32(n) if n >= 0 => visitor.visit_u64(n as u64),
            UbjsonValue::Int64(n) if n >= 0 => visitor.visit_u64(n as u64),
            UbjsonValue::HighPrecision(s) => {
                // Try to parse high-precision number as u64
                s.parse::<u64>()
                    .map_err(|_| UbjsonError::serde(format!("Cannot parse high-precision number as u64: {}", s)))
                    .and_then(|n| visitor.visit_u64(n))
            }
            _ => Err(UbjsonError::serde(format!("Expected u64, found {}", value.type_name()))),
        }
    }

    fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::Float32(f) => visitor.visit_f32(f),
            UbjsonValue::Float64(f) => visitor.visit_f32(f as f32),
            UbjsonValue::Int8(n) => visitor.visit_f32(n as f32),
            UbjsonValue::UInt8(n) => visitor.visit_f32(n as f32),
            UbjsonValue::Int16(n) => visitor.visit_f32(n as f32),
            UbjsonValue::Int32(n) => visitor.visit_f32(n as f32),
            UbjsonValue::Int64(n) => visitor.visit_f32(n as f32),
            _ => Err(UbjsonError::serde(format!("Expected f32, found {}", value.type_name()))),
        }
    }

    fn deserialize_f64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::Float32(f) => visitor.visit_f64(f as f64),
            UbjsonValue::Float64(f) => visitor.visit_f64(f),
            UbjsonValue::Int8(n) => visitor.visit_f64(n as f64),
            UbjsonValue::UInt8(n) => visitor.visit_f64(n as f64),
            UbjsonValue::Int16(n) => visitor.visit_f64(n as f64),
            UbjsonValue::Int32(n) => visitor.visit_f64(n as f64),
            UbjsonValue::Int64(n) => visitor.visit_f64(n as f64),
            UbjsonValue::HighPrecision(s) => {
                // Try to parse high-precision number as f64
                s.parse::<f64>()
                    .map_err(|_| UbjsonError::serde(format!("Cannot parse high-precision number as f64: {}", s)))
                    .and_then(|f| visitor.visit_f64(f))
            }
            _ => Err(UbjsonError::serde(format!("Expected f64, found {}", value.type_name()))),
        }
    }

    fn deserialize_char<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::Char(c) => visitor.visit_char(c),
            UbjsonValue::String(s) => {
                let mut chars = s.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => visitor.visit_char(c),
                    _ => Err(UbjsonError::serde("String must contain exactly one character to deserialize as char")),
                }
            }
            _ => Err(UbjsonError::serde(format!("Expected char, found {}", value.type_name()))),
        }
    }

    fn deserialize_str<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::String(s) => visitor.visit_string(s),
            UbjsonValue::Char(c) => visitor.visit_string(c.to_string()),
            _ => Err(UbjsonError::serde(format!("Expected string, found {}", value.type_name()))),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::Array(arr) => {
                // Convert array of UInt8 values to bytes
                let mut bytes = Vec::with_capacity(arr.len());
                for element in arr {
                    match element {
                        UbjsonValue::UInt8(b) => bytes.push(b),
                        UbjsonValue::Int8(b) if b >= 0 => bytes.push(b as u8),
                        _ => return Err(UbjsonError::serde("Array elements must be bytes (0-255) to deserialize as bytes")),
                    }
                }
                visitor.visit_byte_buf(bytes)
            }
            _ => Err(UbjsonError::serde(format!("Expected array of bytes, found {}", value.type_name()))),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::Null => visitor.visit_none(),
            _ => visitor.visit_some(UbjsonValueDeserializer::new(value)),
        }
    }

    fn deserialize_unit<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::Null => visitor.visit_unit(),
            _ => Err(UbjsonError::serde(format!("Expected null for unit, found {}", value.type_name()))),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::Array(arr) | 
            UbjsonValue::StronglyTypedArray { elements: arr, .. } => {
                visitor.visit_seq(SeqDeserializer::new(arr))
            }
            _ => Err(UbjsonError::serde(format!("Expected array, found {}", value.type_name()))),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::Object(obj) | 
            UbjsonValue::StronglyTypedObject { pairs: obj, .. } => {
                visitor.visit_map(MapDeserializer::new(obj))
            }
            _ => Err(UbjsonError::serde(format!("Expected object, found {}", value.type_name()))),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        mut self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.deserialize_value()?;
        match value {
            UbjsonValue::String(variant) => {
                // Unit variant
                visitor.visit_enum(variant.into_deserializer())
            }
            UbjsonValue::Object(mut obj) => {
                if obj.len() == 1 {
                    let (variant, value) = obj.drain().next().unwrap();
                    visitor.visit_enum(EnumDeserializer::new(variant, value))
                } else {
                    Err(UbjsonError::serde("Enum object must have exactly one key-value pair"))
                }
            }
            _ => Err(UbjsonError::serde(format!("Expected string or object for enum, found {}", value.type_name()))),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

#[cfg(feature = "serde")]
impl<R: Read> UbjsonDeserializer<R> {
    fn deserialize_ubjson_value<'de, V>(&self, value: UbjsonValue, visitor: V) -> Result<V::Value, UbjsonError>
    where
        V: de::Visitor<'de>,
    {
        match value {
            UbjsonValue::Null => visitor.visit_unit(),
            UbjsonValue::Bool(b) => visitor.visit_bool(b),
            UbjsonValue::Int8(n) => visitor.visit_i8(n),
            UbjsonValue::UInt8(n) => visitor.visit_u8(n),
            UbjsonValue::Int16(n) => visitor.visit_i16(n),
            UbjsonValue::Int32(n) => visitor.visit_i32(n),
            UbjsonValue::Int64(n) => visitor.visit_i64(n),
            UbjsonValue::Float32(f) => visitor.visit_f32(f),
            UbjsonValue::Float64(f) => visitor.visit_f64(f),
            UbjsonValue::HighPrecision(s) => visitor.visit_string(s),
            UbjsonValue::Char(c) => visitor.visit_char(c),
            UbjsonValue::String(s) => visitor.visit_string(s),
            UbjsonValue::Array(arr) => visitor.visit_seq(SeqDeserializer::new(arr)),
            UbjsonValue::Object(obj) => visitor.visit_map(MapDeserializer::new(obj)),
            UbjsonValue::StronglyTypedArray { elements, .. } => visitor.visit_seq(SeqDeserializer::new(elements)),
            UbjsonValue::StronglyTypedObject { pairs, .. } => visitor.visit_map(MapDeserializer::new(pairs)),
        }
    }
}

// Helper deserializers for complex types
#[cfg(feature = "serde")]
struct UbjsonValueDeserializer {
    value: UbjsonValue,
}

#[cfg(feature = "serde")]
impl UbjsonValueDeserializer {
    fn new(value: UbjsonValue) -> Self {
        Self { value }
    }
}

#[cfg(feature = "serde")]
impl<'de> de::Deserializer<'de> for UbjsonValueDeserializer {
    type Error = UbjsonError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            UbjsonValue::Null => visitor.visit_unit(),
            UbjsonValue::Bool(b) => visitor.visit_bool(b),
            UbjsonValue::Int8(n) => visitor.visit_i8(n),
            UbjsonValue::UInt8(n) => visitor.visit_u8(n),
            UbjsonValue::Int16(n) => visitor.visit_i16(n),
            UbjsonValue::Int32(n) => visitor.visit_i32(n),
            UbjsonValue::Int64(n) => visitor.visit_i64(n),
            UbjsonValue::Float32(f) => visitor.visit_f32(f),
            UbjsonValue::Float64(f) => visitor.visit_f64(f),
            UbjsonValue::HighPrecision(s) => visitor.visit_string(s),
            UbjsonValue::Char(c) => visitor.visit_char(c),
            UbjsonValue::String(s) => visitor.visit_string(s),
            UbjsonValue::Array(arr) => visitor.visit_seq(SeqDeserializer::new(arr)),
            UbjsonValue::Object(mut obj) => {
                // Check if this could be an enum (object with exactly one key-value pair)
                if obj.len() == 1 {
                    let (variant, value) = obj.drain().next().unwrap();
                    visitor.visit_enum(EnumDeserializer::new(variant, value))
                } else {
                    visitor.visit_map(MapDeserializer::new(obj))
                }
            }
            UbjsonValue::StronglyTypedArray { elements, .. } => visitor.visit_seq(SeqDeserializer::new(elements)),
            UbjsonValue::StronglyTypedObject { pairs, .. } => visitor.visit_map(MapDeserializer::new(pairs)),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[cfg(feature = "serde")]
struct SeqDeserializer {
    elements: std::vec::IntoIter<UbjsonValue>,
}

#[cfg(feature = "serde")]
impl SeqDeserializer {
    fn new(elements: Vec<UbjsonValue>) -> Self {
        Self {
            elements: elements.into_iter(),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> de::SeqAccess<'de> for SeqDeserializer {
    type Error = UbjsonError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.elements.next() {
            Some(value) => seed.deserialize(UbjsonValueDeserializer::new(value)).map(Some),
            None => Ok(None),
        }
    }
}

#[cfg(feature = "serde")]
struct MapDeserializer {
    entries: std::collections::hash_map::IntoIter<String, UbjsonValue>,
    current_value: Option<UbjsonValue>,
}

#[cfg(feature = "serde")]
impl MapDeserializer {
    fn new(map: std::collections::HashMap<String, UbjsonValue>) -> Self {
        Self {
            entries: map.into_iter(),
            current_value: None,
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> de::MapAccess<'de> for MapDeserializer {
    type Error = UbjsonError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.entries.next() {
            Some((key, value)) => {
                self.current_value = Some(value);
                seed.deserialize(key.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.current_value.take() {
            Some(value) => seed.deserialize(UbjsonValueDeserializer::new(value)),
            None => Err(UbjsonError::serde("next_value_seed called without next_key_seed")),
        }
    }
}

#[cfg(feature = "serde")]
struct EnumDeserializer {
    variant: String,
    value: UbjsonValue,
}

#[cfg(feature = "serde")]
impl EnumDeserializer {
    fn new(variant: String, value: UbjsonValue) -> Self {
        Self { variant, value }
    }
}

#[cfg(feature = "serde")]
impl<'de> de::EnumAccess<'de> for EnumDeserializer {
    type Error = UbjsonError;
    type Variant = UbjsonValueDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(self.variant.into_deserializer())?;
        Ok((variant, UbjsonValueDeserializer::new(self.value)))
    }
}

#[cfg(feature = "serde")]
impl<'de> de::VariantAccess<'de> for UbjsonValueDeserializer {
    type Error = UbjsonError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            UbjsonValue::Null => Ok(()),
            _ => Err(UbjsonError::serde("Expected null for unit variant")),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            UbjsonValue::Array(arr) => visitor.visit_seq(SeqDeserializer::new(arr)),
            _ => Err(UbjsonError::serde("Expected array for tuple variant")),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            UbjsonValue::Object(obj) => visitor.visit_map(MapDeserializer::new(obj)),
            _ => Err(UbjsonError::serde("Expected object for struct variant")),
        }
    }
}

// Helper trait for string deserialization
#[cfg(feature = "serde")]
trait StringDeserializer {
    fn into_deserializer(self) -> de::value::StringDeserializer<UbjsonError>;
}

#[cfg(feature = "serde")]
impl StringDeserializer for String {
    fn into_deserializer(self) -> de::value::StringDeserializer<UbjsonError> {
        de::value::StringDeserializer::new(self)
    }
}