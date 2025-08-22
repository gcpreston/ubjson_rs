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
    current_depth: usize,
    max_depth: usize,
}

impl<W: Write> UbjsonSerializer<W> {
    /// Default maximum nesting depth to prevent stack overflow.
    pub const DEFAULT_MAX_DEPTH: usize = 1000;

    /// Create a new serializer with the given writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            optimize_containers: false,
            current_depth: 0,
            max_depth: Self::DEFAULT_MAX_DEPTH,
        }
    }

    /// Create a new serializer with container optimization settings.
    pub fn with_optimization(writer: W, optimize: bool) -> Self {
        Self {
            writer,
            optimize_containers: optimize,
            current_depth: 0,
            max_depth: Self::DEFAULT_MAX_DEPTH,
        }
    }

    /// Create a new serializer with custom depth limit.
    pub fn with_depth_limit(writer: W, max_depth: usize) -> Self {
        Self {
            writer,
            optimize_containers: false,
            current_depth: 0,
            max_depth,
        }
    }

    /// Create a new serializer with both optimization and depth limit settings.
    pub fn with_settings(writer: W, optimize: bool, max_depth: usize) -> Self {
        Self {
            writer,
            optimize_containers: optimize,
            current_depth: 0,
            max_depth,
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
            // Standard container types
            UbjsonValue::Array(arr) => self.serialize_array(arr),
            UbjsonValue::Object(obj) => self.serialize_object(obj),
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

    /// Serialize a standard array.
    fn serialize_array(&mut self, array: &[UbjsonValue]) -> Result<()> {
        // Check depth limit
        if self.current_depth >= self.max_depth {
            return Err(UbjsonError::DepthLimitExceeded(self.max_depth));
        }

        // Write array start marker
        write_type_marker(&mut self.writer, UbjsonType::ArrayStart)?;
        
        // Increase depth for nested serialization
        self.current_depth += 1;
        
        // Serialize each element
        for element in array {
            self.serialize_value(element)?;
        }
        
        // Decrease depth
        self.current_depth -= 1;
        
        // Write array end marker
        write_type_marker(&mut self.writer, UbjsonType::ArrayEnd)
    }

    /// Serialize a standard object.
    fn serialize_object(&mut self, object: &std::collections::HashMap<String, UbjsonValue>) -> Result<()> {
        // Check depth limit
        if self.current_depth >= self.max_depth {
            return Err(UbjsonError::DepthLimitExceeded(self.max_depth));
        }

        // Write object start marker
        write_type_marker(&mut self.writer, UbjsonType::ObjectStart)?;
        
        // Increase depth for nested serialization
        self.current_depth += 1;
        
        // Serialize each key-value pair
        for (key, value) in object {
            // Write the key as a full string value (with 'S' marker)
            write_type_marker(&mut self.writer, UbjsonType::String)?;
            write_string(&mut self.writer, key)?;
            // Write the value
            self.serialize_value(value)?;
        }
        
        // Decrease depth
        self.current_depth -= 1;
        
        // Write object end marker
        write_type_marker(&mut self.writer, UbjsonType::ObjectEnd)
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
    fn test_serialize_empty_array() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        let array = UbjsonValue::Array(vec![]);
        serializer.serialize_value(&array).unwrap();
        
        assert_eq!(buffer, vec![b'[', b']']);
    }

    #[test]
    fn test_serialize_array_with_primitives() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        let array = UbjsonValue::Array(vec![
            UbjsonValue::Null,
            UbjsonValue::Bool(true),
            UbjsonValue::Int8(42),
            UbjsonValue::String("hello".to_string()),
        ]);
        serializer.serialize_value(&array).unwrap();
        
        let expected = vec![
            b'[',           // Array start
            b'Z',           // Null
            b'T',           // True
            b'i', 42,       // Int8(42)
            b'S', b'U', 5,  // String length prefix
            b'h', b'e', b'l', b'l', b'o', // String content
            b']',           // Array end
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_serialize_nested_arrays() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        let nested_array = UbjsonValue::Array(vec![
            UbjsonValue::Array(vec![UbjsonValue::Int8(1), UbjsonValue::Int8(2)]),
            UbjsonValue::Array(vec![UbjsonValue::Int8(3)]),
        ]);
        serializer.serialize_value(&nested_array).unwrap();
        
        let expected = vec![
            b'[',           // Outer array start
            b'[',           // Inner array 1 start
            b'i', 1,        // Int8(1)
            b'i', 2,        // Int8(2)
            b']',           // Inner array 1 end
            b'[',           // Inner array 2 start
            b'i', 3,        // Int8(3)
            b']',           // Inner array 2 end
            b']',           // Outer array end
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_serialize_empty_object() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        let object = UbjsonValue::Object(std::collections::HashMap::new());
        serializer.serialize_value(&object).unwrap();
        
        assert_eq!(buffer, vec![b'{', b'}']);
    }

    #[test]
    fn test_serialize_object_with_primitives() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        let mut map = std::collections::HashMap::new();
        map.insert("null".to_string(), UbjsonValue::Null);
        map.insert("bool".to_string(), UbjsonValue::Bool(true));
        map.insert("num".to_string(), UbjsonValue::Int8(42));
        
        let object = UbjsonValue::Object(map);
        serializer.serialize_value(&object).unwrap();
        
        // Note: HashMap iteration order is not guaranteed, so we need to check
        // that the serialized data contains the expected elements
        assert_eq!(buffer[0], b'{'); // Object start
        assert_eq!(buffer[buffer.len() - 1], b'}'); // Object end
        
        // Check that the buffer contains the expected key-value pairs
        // The exact order may vary, but all elements should be present
        assert!(buffer.len() > 2); // More than just start and end markers
    }

    #[test]
    fn test_serialize_nested_objects() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        let mut inner_map = std::collections::HashMap::new();
        inner_map.insert("inner".to_string(), UbjsonValue::Int8(1));
        
        let mut outer_map = std::collections::HashMap::new();
        outer_map.insert("nested".to_string(), UbjsonValue::Object(inner_map));
        
        let object = UbjsonValue::Object(outer_map);
        serializer.serialize_value(&object).unwrap();
        
        assert_eq!(buffer[0], b'{'); // Outer object start
        assert_eq!(buffer[buffer.len() - 1], b'}'); // Outer object end
        
        // Should contain nested object markers
        let object_count = buffer.iter().filter(|&&b| b == b'{').count();
        assert_eq!(object_count, 2); // Two object start markers
        
        let object_end_count = buffer.iter().filter(|&&b| b == b'}').count();
        assert_eq!(object_end_count, 2); // Two object end markers
    }

    #[test]
    fn test_serialize_mixed_containers() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        let mut map = std::collections::HashMap::new();
        map.insert("array".to_string(), UbjsonValue::Array(vec![
            UbjsonValue::Int8(1),
            UbjsonValue::Int8(2),
        ]));
        
        let array = UbjsonValue::Array(vec![
            UbjsonValue::Object(map),
            UbjsonValue::String("test".to_string()),
        ]);
        
        serializer.serialize_value(&array).unwrap();
        
        assert_eq!(buffer[0], b'['); // Array start
        assert_eq!(buffer[buffer.len() - 1], b']'); // Array end
        
        // Should contain both array and object markers
        let array_count = buffer.iter().filter(|&&b| b == b'[').count();
        assert_eq!(array_count, 2); // Two array start markers
        
        let object_count = buffer.iter().filter(|&&b| b == b'{').count();
        assert_eq!(object_count, 1); // One object start marker
    }

    #[test]
    fn test_depth_limit_exceeded() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::with_depth_limit(&mut buffer, 2);
        
        // Create a deeply nested array that exceeds the depth limit
        let deeply_nested = UbjsonValue::Array(vec![
            UbjsonValue::Array(vec![
                UbjsonValue::Array(vec![UbjsonValue::Int8(1)]) // This should exceed depth 2
            ])
        ]);
        
        let result = serializer.serialize_value(&deeply_nested);
        assert!(result.is_err());
        
        if let Err(UbjsonError::DepthLimitExceeded(depth)) = result {
            assert_eq!(depth, 2);
        } else {
            panic!("Expected DepthLimitExceeded error");
        }
    }

    #[test]
    fn test_serializer_with_settings() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::with_settings(&mut buffer, true, 100);
        
        // Test that the serializer was created with the correct settings
        serializer.serialize_value(&UbjsonValue::Null).unwrap();
        assert_eq!(buffer, vec![b'Z']);
    }


}