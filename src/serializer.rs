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
    write_float32, write_float64, write_string, write_char, write_length
};
use crate::types::optimization::{TYPE_MARKER, COUNT_MARKER};

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
            // Optimized container types
            UbjsonValue::StronglyTypedArray { element_type, count, elements } => {
                self.serialize_strongly_typed_array(*element_type, *count, elements)
            }
            UbjsonValue::StronglyTypedObject { value_type, count, pairs } => {
                self.serialize_strongly_typed_object(*value_type, *count, pairs)
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

        // Check if optimization is enabled and array is homogeneous
        if self.optimize_containers && !array.is_empty() {
            if let Some(element_type) = self.detect_homogeneous_array_type(array) {
                return self.serialize_strongly_typed_array(element_type, Some(array.len()), array);
            }
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

        // Check if optimization is enabled and object is homogeneous
        if self.optimize_containers && !object.is_empty() {
            if let Some(value_type) = self.detect_homogeneous_object_type(object) {
                return self.serialize_strongly_typed_object(value_type, Some(object.len()), object);
            }
        }

        // Write object start marker
        write_type_marker(&mut self.writer, UbjsonType::ObjectStart)?;
        
        // Increase depth for nested serialization
        self.current_depth += 1;
        
        // Serialize each key-value pair
        for (key, value) in object {
            // Write the key as a raw string (without 'S' marker per UBJSON spec)
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

    /// Detect if an array is homogeneous and return the common element type.
    fn detect_homogeneous_array_type(&self, array: &[UbjsonValue]) -> Option<UbjsonType> {
        if array.is_empty() {
            return None;
        }

        let first_type = array[0].get_type();
        
        // Only optimize primitive types (not containers)
        if !first_type.is_primitive() {
            return None;
        }

        // Check if all elements have the same type
        for element in array.iter().skip(1) {
            if element.get_type() != first_type {
                return None;
            }
        }

        Some(first_type)
    }

    /// Detect if an object is homogeneous and return the common value type.
    fn detect_homogeneous_object_type(&self, object: &std::collections::HashMap<String, UbjsonValue>) -> Option<UbjsonType> {
        if object.is_empty() {
            return None;
        }

        let mut values = object.values();
        let first_type = values.next()?.get_type();
        
        // Only optimize primitive types (not containers)
        if !first_type.is_primitive() {
            return None;
        }

        // Check if all values have the same type
        for value in values {
            if value.get_type() != first_type {
                return None;
            }
        }

        Some(first_type)
    }

    /// Serialize a strongly-typed array with optimization markers.
    fn serialize_strongly_typed_array(
        &mut self,
        element_type: UbjsonType,
        count: Option<usize>,
        elements: &[UbjsonValue],
    ) -> Result<()> {
        // Check depth limit
        if self.current_depth >= self.max_depth {
            return Err(UbjsonError::DepthLimitExceeded(self.max_depth));
        }

        // Write array start marker
        write_type_marker(&mut self.writer, UbjsonType::ArrayStart)?;
        
        // Write type optimization marker '$'
        self.writer.write_all(&[TYPE_MARKER])?;
        
        // Write the element type
        write_type_marker(&mut self.writer, element_type)?;
        
        // Write count optimization if provided
        if let Some(count) = count {
            self.writer.write_all(&[COUNT_MARKER])?;
            write_length(&mut self.writer, count)?;
        }
        
        // Increase depth for nested serialization
        self.current_depth += 1;
        
        // Serialize elements without type markers (since type is already specified)
        for element in elements {
            self.serialize_value_without_type_marker(element, element_type)?;
        }
        
        // Decrease depth
        self.current_depth -= 1;
        
        // Write array end marker (only if count was not provided)
        if count.is_none() {
            write_type_marker(&mut self.writer, UbjsonType::ArrayEnd)?;
        }
        
        Ok(())
    }

    /// Serialize a strongly-typed object with optimization markers.
    fn serialize_strongly_typed_object(
        &mut self,
        value_type: UbjsonType,
        count: Option<usize>,
        pairs: &std::collections::HashMap<String, UbjsonValue>,
    ) -> Result<()> {
        // Check depth limit
        if self.current_depth >= self.max_depth {
            return Err(UbjsonError::DepthLimitExceeded(self.max_depth));
        }

        // Write object start marker
        write_type_marker(&mut self.writer, UbjsonType::ObjectStart)?;
        
        // Write type optimization marker '$'
        self.writer.write_all(&[TYPE_MARKER])?;
        
        // Write the value type
        write_type_marker(&mut self.writer, value_type)?;
        
        // Write count optimization if provided
        if let Some(count) = count {
            self.writer.write_all(&[COUNT_MARKER])?;
            write_length(&mut self.writer, count)?;
        }
        
        // Increase depth for nested serialization
        self.current_depth += 1;
        
        // Serialize key-value pairs without value type markers
        for (key, value) in pairs {
            // Write the key as a raw string (without 'S' marker per UBJSON spec)
            write_string(&mut self.writer, key)?;
            // Write the value without type marker (since type is already specified)
            self.serialize_value_without_type_marker(value, value_type)?;
        }
        
        // Decrease depth
        self.current_depth -= 1;
        
        // Write object end marker (only if count was not provided)
        if count.is_none() {
            write_type_marker(&mut self.writer, UbjsonType::ObjectEnd)?;
        }
        
        Ok(())
    }

    /// Serialize a value without its type marker (for optimized containers).
    fn serialize_value_without_type_marker(&mut self, value: &UbjsonValue, expected_type: UbjsonType) -> Result<()> {
        // Verify the value matches the expected type
        if value.get_type() != expected_type {
            return Err(UbjsonError::invalid_format(format!(
                "Value type {} does not match expected type {}",
                value.get_type(),
                expected_type
            )));
        }

        match value {
            UbjsonValue::Null => Ok(()), // No data to write for null
            UbjsonValue::Bool(true) => Ok(()), // No data to write for true
            UbjsonValue::Bool(false) => Ok(()), // No data to write for false
            UbjsonValue::Int8(n) => write_int8(&mut self.writer, *n),
            UbjsonValue::UInt8(n) => write_uint8(&mut self.writer, *n),
            UbjsonValue::Int16(n) => write_int16(&mut self.writer, *n),
            UbjsonValue::Int32(n) => write_int32(&mut self.writer, *n),
            UbjsonValue::Int64(n) => write_int64(&mut self.writer, *n),
            UbjsonValue::Float32(n) => write_float32(&mut self.writer, *n),
            UbjsonValue::Float64(n) => write_float64(&mut self.writer, *n),
            UbjsonValue::HighPrecision(s) => write_string(&mut self.writer, s),
            UbjsonValue::Char(c) => write_char(&mut self.writer, *c),
            UbjsonValue::String(s) => write_string(&mut self.writer, s),
            // Containers should not be in optimized containers (only primitives)
            _ => Err(UbjsonError::invalid_format(
                "Container types cannot be used in optimized containers"
            )),
        }
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

    #[test]
    fn test_serialize_homogeneous_int8_array_with_optimization() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
        
        let array = UbjsonValue::Array(vec![
            UbjsonValue::Int8(1),
            UbjsonValue::Int8(2),
            UbjsonValue::Int8(3),
        ]);
        serializer.serialize_value(&array).unwrap();
        
        let expected = vec![
            b'[',           // Array start
            b'$',           // Type marker
            b'i',           // Int8 type
            b'#',           // Count marker
            b'U', 3,        // Count (3 as uint8)
            1, 2, 3,        // Elements without type markers
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_serialize_homogeneous_string_array_with_optimization() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
        
        let array = UbjsonValue::Array(vec![
            UbjsonValue::String("hello".to_string()),
            UbjsonValue::String("world".to_string()),
        ]);
        serializer.serialize_value(&array).unwrap();
        
        let expected = vec![
            b'[',           // Array start
            b'$',           // Type marker
            b'S',           // String type
            b'#',           // Count marker
            b'U', 2,        // Count (2 as uint8)
            b'U', 5,        // Length of "hello"
            b'h', b'e', b'l', b'l', b'o', // "hello"
            b'U', 5,        // Length of "world"
            b'w', b'o', b'r', b'l', b'd', // "world"
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_serialize_heterogeneous_array_without_optimization() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
        
        let array = UbjsonValue::Array(vec![
            UbjsonValue::Int8(1),
            UbjsonValue::String("hello".to_string()),
            UbjsonValue::Bool(true),
        ]);
        serializer.serialize_value(&array).unwrap();
        
        // Should fall back to standard array format
        let expected = vec![
            b'[',           // Array start
            b'i', 1,        // Int8(1)
            b'S', b'U', 5,  // String length prefix
            b'h', b'e', b'l', b'l', b'o', // String content
            b'T',           // True
            b']',           // Array end
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_serialize_homogeneous_object_with_optimization() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
        
        let mut map = std::collections::HashMap::new();
        map.insert("a".to_string(), UbjsonValue::Int32(100));
        map.insert("b".to_string(), UbjsonValue::Int32(200));
        
        let object = UbjsonValue::Object(map);
        serializer.serialize_value(&object).unwrap();
        
        // Check that optimization markers are present
        assert_eq!(buffer[0], b'{'); // Object start
        assert_eq!(buffer[1], b'$'); // Type marker
        assert_eq!(buffer[2], b'l'); // Int32 type
        assert_eq!(buffer[3], b'#'); // Count marker
        assert_eq!(buffer[4], b'U'); // Count type (uint8)
        assert_eq!(buffer[5], 2);    // Count value (2)
        
        // The rest contains key-value pairs without value type markers
        // Order is not guaranteed with HashMap, so we just check structure
        assert!(buffer.len() > 6);
    }

    #[test]
    fn test_serialize_heterogeneous_object_without_optimization() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
        
        let mut map = std::collections::HashMap::new();
        map.insert("num".to_string(), UbjsonValue::Int32(42));
        map.insert("str".to_string(), UbjsonValue::String("hello".to_string()));
        
        let object = UbjsonValue::Object(map);
        serializer.serialize_value(&object).unwrap();
        
        // Should fall back to standard object format
        assert_eq!(buffer[0], b'{'); // Object start
        assert_eq!(buffer[buffer.len() - 1], b'}'); // Object end
        
        // Should not contain optimization markers
        assert!(!buffer.contains(&b'$'));
        assert!(!buffer.contains(&b'#'));
    }

    #[test]
    fn test_serialize_strongly_typed_array_explicit() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        let array = UbjsonValue::StronglyTypedArray {
            element_type: UbjsonType::Float32,
            count: Some(2),
            elements: vec![
                UbjsonValue::Float32(1.5),
                UbjsonValue::Float32(2.5),
            ],
        };
        serializer.serialize_value(&array).unwrap();
        
        let expected_start = vec![
            b'[',           // Array start
            b'$',           // Type marker
            b'd',           // Float32 type
            b'#',           // Count marker
            b'U', 2,        // Count (2 as uint8)
        ];
        
        // Check the start of the buffer
        assert_eq!(&buffer[0..6], &expected_start[..]);
        
        // Check that float values are present (exact bytes depend on IEEE 754 representation)
        assert_eq!(buffer.len(), 6 + 8); // 6 bytes header + 2 * 4 bytes for floats
    }

    #[test]
    fn test_serialize_strongly_typed_object_explicit() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        let mut pairs = std::collections::HashMap::new();
        pairs.insert("x".to_string(), UbjsonValue::Bool(true));
        pairs.insert("y".to_string(), UbjsonValue::Bool(false));
        
        let object = UbjsonValue::StronglyTypedObject {
            value_type: UbjsonType::True, // Using True as the type (Bool values will be handled specially)
            count: Some(2),
            pairs,
        };
        
        // This should fail because Bool values can't be properly handled in strongly-typed containers
        // since True and False are different types
        let result = serializer.serialize_value(&object);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_strongly_typed_array_without_count() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        let array = UbjsonValue::StronglyTypedArray {
            element_type: UbjsonType::UInt8,
            count: None,
            elements: vec![
                UbjsonValue::UInt8(10),
                UbjsonValue::UInt8(20),
                UbjsonValue::UInt8(30),
            ],
        };
        serializer.serialize_value(&array).unwrap();
        
        let expected = vec![
            b'[',           // Array start
            b'$',           // Type marker
            b'U',           // UInt8 type
            10, 20, 30,     // Elements without type markers
            b']',           // Array end (since no count was provided)
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_serialize_strongly_typed_object_without_count() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        let mut pairs = std::collections::HashMap::new();
        pairs.insert("a".to_string(), UbjsonValue::Int16(100));
        pairs.insert("b".to_string(), UbjsonValue::Int16(200));
        
        let object = UbjsonValue::StronglyTypedObject {
            value_type: UbjsonType::Int16,
            count: None,
            pairs,
        };
        serializer.serialize_value(&object).unwrap();
        
        // Check structure
        assert_eq!(buffer[0], b'{'); // Object start
        assert_eq!(buffer[1], b'$'); // Type marker
        assert_eq!(buffer[2], b'I'); // Int16 type
        assert_eq!(buffer[buffer.len() - 1], b'}'); // Object end
        
        // Should not contain count marker since count was None
        let has_count_marker = buffer.windows(2).any(|w| w == [b'#', b'U'] || w == [b'#', b'I'] || w == [b'#', b'l'] || w == [b'#', b'L']);
        assert!(!has_count_marker);
    }

    #[test]
    fn test_serialize_empty_array_no_optimization() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
        
        let array = UbjsonValue::Array(vec![]);
        serializer.serialize_value(&array).unwrap();
        
        // Empty arrays should not be optimized
        assert_eq!(buffer, vec![b'[', b']']);
    }

    #[test]
    fn test_serialize_empty_object_no_optimization() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
        
        let object = UbjsonValue::Object(std::collections::HashMap::new());
        serializer.serialize_value(&object).unwrap();
        
        // Empty objects should not be optimized
        assert_eq!(buffer, vec![b'{', b'}']);
    }

    #[test]
    fn test_serialize_array_with_containers_no_optimization() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
        
        let array = UbjsonValue::Array(vec![
            UbjsonValue::Array(vec![UbjsonValue::Int8(1)]),
            UbjsonValue::Array(vec![UbjsonValue::Int8(2)]),
        ]);
        serializer.serialize_value(&array).unwrap();
        
        // The outer array containing containers should not be optimized
        assert_eq!(buffer[0], b'['); // Array start
        assert_eq!(buffer[buffer.len() - 1], b']'); // Array end
        
        // The outer array should not have optimization markers immediately after the start
        // (buffer[1] should be '[' for the first inner array, not '$')
        assert_eq!(buffer[1], b'['); // First inner array start
        
        // But the inner arrays can be optimized since they contain homogeneous primitives
        // This is correct behavior - only the outer array should not be optimized
    }

    #[test]
    fn test_serialize_optimization_disabled() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, false);
        
        let array = UbjsonValue::Array(vec![
            UbjsonValue::Int8(1),
            UbjsonValue::Int8(2),
            UbjsonValue::Int8(3),
        ]);
        serializer.serialize_value(&array).unwrap();
        
        // Should use standard format even though array is homogeneous
        let expected = vec![
            b'[',           // Array start
            b'i', 1,        // Int8(1)
            b'i', 2,        // Int8(2)
            b'i', 3,        // Int8(3)
            b']',           // Array end
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_serialize_value_type_mismatch_error() {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        
        // Create a strongly-typed array with mismatched element type
        let array = UbjsonValue::StronglyTypedArray {
            element_type: UbjsonType::Int8,
            count: Some(1),
            elements: vec![UbjsonValue::Int32(42)], // Wrong type!
        };
        
        let result = serializer.serialize_value(&array);
        assert!(result.is_err());
        
        if let Err(UbjsonError::InvalidFormat(msg)) = result {
            assert!(msg.contains("does not match expected type"));
        } else {
            panic!("Expected InvalidFormat error");
        }
    }


}