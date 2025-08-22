//! UBJSON serialization functionality.
//!
//! This module provides the UbjsonSerializer struct for converting Rust values
//! and UbjsonValue instances into UBJSON binary format.

use std::io::Write;
use crate::error::{UbjsonError, Result};
use crate::types::UbjsonType;
use crate::value::UbjsonValue;
use crate::encoding::{
    write_type_marker, write_int8, write_uint8, write_int16, write_int32, write_int64,
    write_float32, write_float64, write_string, write_char
};

/// Serializer for converting values to UBJSON binary format.
pub struct UbjsonSerializer<W: Write> {
    writer: W,
    optimize_containers: bool,
}

impl<W: Write> UbjsonSerializer<W> {
    /// Create a new serializer with the given writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            optimize_containers: false,
        }
    }

    /// Create a new serializer with container optimization settings.
    pub fn with_optimization(writer: W, optimize: bool) -> Self {
        Self {
            writer,
            optimize_containers: optimize,
        }
    }

    /// Serialize a UbjsonValue to the writer.
    pub fn serialize_value(&mut self, value: &UbjsonValue) -> Result<()> {
        match value {
            UbjsonValue::Null => self.serialize_null(),
            UbjsonValue::Bool(b) => self.serialize_bool(*b),
            UbjsonValue::Int8(n) => self.serialize_int8(*n),
            UbjsonValue::UInt8(n) => self.serialize_uint8(*n),
            UbjsonValue::Int16(n) => self.serialize_int16(*n),
            UbjsonValue::Int32(n) => self.serialize_int32(*n),
            UbjsonValue::Int64(n) => self.serialize_int64(*n),
            UbjsonValue::Float32(n) => self.serialize_float32(*n),
            UbjsonValue::Float64(n) => self.serialize_float64(*n),
            UbjsonValue::HighPrecision(s) => self.serialize_high_precision(s),
            UbjsonValue::Char(c) => self.serialize_char(*c),
            UbjsonValue::String(s) => self.serialize_string(s),
            // Container types will be implemented in later tasks
            UbjsonValue::Array(_) => {
                Err(UbjsonError::unsupported_type("Array serialization not yet implemented"))
            }
            UbjsonValue::Object(_) => {
                Err(UbjsonError::unsupported_type("Object serialization not yet implemented"))
            }
            UbjsonValue::StronglyTypedArray { .. } => {
                Err(UbjsonError::unsupported_type("StronglyTypedArray serialization not yet implemented"))
            }
            UbjsonValue::StronglyTypedObject { .. } => {
                Err(UbjsonError::unsupported_type("StronglyTypedObject serialization not yet implemented"))
            }
        }
    }

    /// Serialize a null value.
    fn serialize_null(&mut self) -> Result<()> {
        write_type_marker(&mut self.writer, UbjsonType::Null)
    }

    /// Serialize a boolean value.
    fn serialize_bool(&mut self, value: bool) -> Result<()> {
        let type_marker = if value {
            UbjsonType::True
        } else {
            UbjsonType::False
        };
        write_type_marker(&mut self.writer, type_marker)
    }

    /// Serialize a signed 8-bit integer.
    fn serialize_int8(&mut self, value: i8) -> Result<()> {
        write_type_marker(&mut self.writer, UbjsonType::Int8)?;
        write_int8(&mut self.writer, value)
    }

    /// Serialize an unsigned 8-bit integer.
    fn serialize_uint8(&mut self, value: u8) -> Result<()> {
        write_type_marker(&mut self.writer, UbjsonType::UInt8)?;
        write_uint8(&mut self.writer, value)
    }

    /// Serialize a signed 16-bit integer.
    fn serialize_int16(&mut self, value: i16) -> Result<()> {
        write_type_marker(&mut self.writer, UbjsonType::Int16)?;
        write_int16(&mut self.writer, value)
    }

    /// Serialize a signed 32-bit integer.
    fn serialize_int32(&mut self, value: i32) -> Result<()> {
        write_type_marker(&mut self.writer, UbjsonType::Int32)?;
        write_int32(&mut self.writer, value)
    }

    /// Serialize a signed 64-bit integer.
    fn serialize_int64(&mut self, value: i64) -> Result<()> {
        write_type_marker(&mut self.writer, UbjsonType::Int64)?;
        write_int64(&mut self.writer, value)
    }

    /// Serialize a 32-bit floating-point number.
    fn serialize_float32(&mut self, value: f32) -> Result<()> {
        write_type_marker(&mut self.writer, UbjsonType::Float32)?;
        write_float32(&mut self.writer, value)
    }

    /// Serialize a 64-bit floating-point number.
    fn serialize_float64(&mut self, value: f64) -> Result<()> {
        write_type_marker(&mut self.writer, UbjsonType::Float64)?;
        write_float64(&mut self.writer, value)
    }

    /// Serialize a high-precision number.
    fn serialize_high_precision(&mut self, value: &str) -> Result<()> {
        write_type_marker(&mut self.writer, UbjsonType::HighPrecision)?;
        write_string(&mut self.writer, value)
    }

    /// Serialize a character.
    fn serialize_char(&mut self, value: char) -> Result<()> {
        write_type_marker(&mut self.writer, UbjsonType::Char)?;
        write_char(&mut self.writer, value)
    }

    /// Serialize a string.
    fn serialize_string(&mut self, value: &str) -> Result<()> {
        write_type_marker(&mut self.writer, UbjsonType::String)?;
        write_string(&mut self.writer, value)
    }

    /// Get a reference to the underlying writer.
    pub fn writer(&self) -> &W {
        &self.writer
    }

    /// Get a mutable reference to the underlying writer.
    pub fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Consume the serializer and return the underlying writer.
    pub fn into_writer(self) -> W {
        self.writer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_serialize_null() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        serializer.serialize_value(&UbjsonValue::Null).unwrap();
        
        assert_eq!(buffer, vec![b'Z']);
    }

    #[test]
    fn test_serialize_bool_true() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        serializer.serialize_value(&UbjsonValue::Bool(true)).unwrap();
        
        assert_eq!(buffer, vec![b'T']);
    }

    #[test]
    fn test_serialize_bool_false() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        serializer.serialize_value(&UbjsonValue::Bool(false)).unwrap();
        
        assert_eq!(buffer, vec![b'F']);
    }

    #[test]
    fn test_serialize_int8() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        serializer.serialize_value(&UbjsonValue::Int8(42)).unwrap();
        
        assert_eq!(buffer, vec![b'i', 42]);
    }

    #[test]
    fn test_serialize_int8_negative() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        serializer.serialize_value(&UbjsonValue::Int8(-42)).unwrap();
        
        assert_eq!(buffer, vec![b'i', 0xD6]); // -42 as u8 in two's complement
    }

    #[test]
    fn test_serialize_uint8() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        serializer.serialize_value(&UbjsonValue::UInt8(255)).unwrap();
        
        assert_eq!(buffer, vec![b'U', 255]);
    }

    #[test]
    fn test_serialize_int16() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        serializer.serialize_value(&UbjsonValue::Int16(1000)).unwrap();
        
        assert_eq!(buffer, vec![b'I', 0x03, 0xE8]); // 1000 in big-endian
    }

    #[test]
    fn test_serialize_int32() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        serializer.serialize_value(&UbjsonValue::Int32(100000)).unwrap();
        
        assert_eq!(buffer, vec![b'l', 0x00, 0x01, 0x86, 0xA0]); // 100000 in big-endian
    }

    #[test]
    fn test_serialize_int64() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        serializer.serialize_value(&UbjsonValue::Int64(1000000000000)).unwrap();
        
        assert_eq!(buffer, vec![b'L', 0x00, 0x00, 0x00, 0xE8, 0xD4, 0xA5, 0x10, 0x00]); // 1000000000000 in big-endian
    }

    #[test]
    fn test_serialize_float32() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        serializer.serialize_value(&UbjsonValue::Float32(3.14159)).unwrap();
        
        // 3.14159 as f32 in big-endian IEEE 754 format
        let expected_bytes = 3.14159f32.to_be_bytes();
        let mut expected = vec![b'd'];
        expected.extend_from_slice(&expected_bytes);
        
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_serialize_float64() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        serializer.serialize_value(&UbjsonValue::Float64(3.141592653589793)).unwrap();
        
        // 3.141592653589793 as f64 in big-endian IEEE 754 format
        let expected_bytes = 3.141592653589793f64.to_be_bytes();
        let mut expected = vec![b'D'];
        expected.extend_from_slice(&expected_bytes);
        
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_serialize_high_precision() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        let high_precision = "3.141592653589793238462643383279502884197";
        serializer.serialize_value(&UbjsonValue::HighPrecision(high_precision.to_string())).unwrap();
        
        let mut expected = vec![b'H', b'U', high_precision.len() as u8];
        expected.extend_from_slice(high_precision.as_bytes());
        
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_serialize_char_ascii() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        serializer.serialize_value(&UbjsonValue::Char('A')).unwrap();
        
        assert_eq!(buffer, vec![b'C', b'A']);
    }

    #[test]
    fn test_serialize_char_unicode() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        serializer.serialize_value(&UbjsonValue::Char('π')).unwrap();
        
        // π in UTF-8 is [0xCF, 0x80]
        assert_eq!(buffer, vec![b'C', 0xCF, 0x80]);
    }

    #[test]
    fn test_serialize_string_empty() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        serializer.serialize_value(&UbjsonValue::String("".to_string())).unwrap();
        
        assert_eq!(buffer, vec![b'S', b'U', 0]); // String marker, length type (uint8), length 0
    }

    #[test]
    fn test_serialize_string_ascii() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        let test_string = "Hello, World!";
        serializer.serialize_value(&UbjsonValue::String(test_string.to_string())).unwrap();
        
        let mut expected = vec![b'S', b'U', test_string.len() as u8];
        expected.extend_from_slice(test_string.as_bytes());
        
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_serialize_string_unicode() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        let test_string = "Hello, 世界!";
        serializer.serialize_value(&UbjsonValue::String(test_string.to_string())).unwrap();
        
        let string_bytes = test_string.as_bytes();
        let mut expected = vec![b'S', b'U', string_bytes.len() as u8];
        expected.extend_from_slice(string_bytes);
        
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_serialize_with_cursor() {
        let mut cursor = Cursor::new(Vec::new());
        let mut serializer = UbjsonSerializer::new(&mut cursor);
        
        serializer.serialize_value(&UbjsonValue::Int32(42)).unwrap();
        
        let buffer = cursor.into_inner();
        assert_eq!(buffer, vec![b'l', 0x00, 0x00, 0x00, 0x2A]); // 42 in big-endian
    }

    #[test]
    fn test_serializer_with_optimization() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
        
        // Optimization setting doesn't affect primitive serialization
        serializer.serialize_value(&UbjsonValue::Bool(true)).unwrap();
        
        assert_eq!(buffer, vec![b'T']);
    }

    #[test]
    fn test_serializer_writer_access() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        // Test writer access methods
        serializer.serialize_value(&UbjsonValue::Null).unwrap();
        
        // Test that we can access the writer
        let _writer_ref = serializer.writer();
        let _writer_mut = serializer.writer_mut();
        
        // Test consuming the serializer
        let _writer = serializer.into_writer();
    }

    #[test]
    fn test_unsupported_container_types() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        // Test that container types return appropriate errors
        let array = UbjsonValue::Array(vec![]);
        let result = serializer.serialize_value(&array);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Array serialization not yet implemented"));
        
        let object = UbjsonValue::Object(std::collections::HashMap::new());
        let result = serializer.serialize_value(&object);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Object serialization not yet implemented"));
    }
}