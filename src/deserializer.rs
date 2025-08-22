//! UBJSON deserialization functionality.
//!
//! This module provides the UbjsonDeserializer struct for reading UBJSON binary data
//! and converting it back to UbjsonValue instances or Rust data structures.

use std::io::Read;
use crate::encoding::{
    read_type_marker, read_int8, read_uint8, read_int16, read_int32, read_int64,
    read_float32, read_float64, read_string, read_char
};
use crate::error::{UbjsonError, Result};
use crate::types::UbjsonType;
use crate::value::UbjsonValue;

/// Deserializer for UBJSON binary data.
pub struct UbjsonDeserializer<R: Read> {
    reader: R,
    max_depth: usize,
    max_size: usize,
    current_depth: usize,
}

impl<R: Read> UbjsonDeserializer<R> {
    /// Create a new deserializer with default limits.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            max_depth: 1000,  // Default depth limit to prevent stack overflow
            max_size: 1_000_000,  // Default size limit to prevent DoS attacks
            current_depth: 0,
        }
    }

    /// Create a new deserializer with custom limits.
    pub fn with_limits(reader: R, max_depth: usize, max_size: usize) -> Self {
        Self {
            reader,
            max_depth,
            max_size,
            current_depth: 0,
        }
    }

    /// Deserialize a single UBJSON value from the reader.
    pub fn deserialize_value(&mut self) -> Result<UbjsonValue> {
        // Check depth limit
        if self.current_depth >= self.max_depth {
            return Err(UbjsonError::DepthLimitExceeded(self.max_depth));
        }

        let type_marker = read_type_marker(&mut self.reader)?;
        self.deserialize_value_with_type(type_marker)
    }

    /// Deserialize a value when the type marker is already known.
    fn deserialize_value_with_type(&mut self, type_marker: UbjsonType) -> Result<UbjsonValue> {
        match type_marker {
            UbjsonType::Null => Ok(UbjsonValue::Null),
            UbjsonType::True => Ok(UbjsonValue::Bool(true)),
            UbjsonType::False => Ok(UbjsonValue::Bool(false)),
            UbjsonType::Int8 => {
                let value = read_int8(&mut self.reader)?;
                Ok(UbjsonValue::Int8(value))
            }
            UbjsonType::UInt8 => {
                let value = read_uint8(&mut self.reader)?;
                Ok(UbjsonValue::UInt8(value))
            }
            UbjsonType::Int16 => {
                let value = read_int16(&mut self.reader)?;
                Ok(UbjsonValue::Int16(value))
            }
            UbjsonType::Int32 => {
                let value = read_int32(&mut self.reader)?;
                Ok(UbjsonValue::Int32(value))
            }
            UbjsonType::Int64 => {
                let value = read_int64(&mut self.reader)?;
                Ok(UbjsonValue::Int64(value))
            }
            UbjsonType::Float32 => {
                let value = read_float32(&mut self.reader)?;
                Ok(UbjsonValue::Float32(value))
            }
            UbjsonType::Float64 => {
                let value = read_float64(&mut self.reader)?;
                Ok(UbjsonValue::Float64(value))
            }
            UbjsonType::HighPrecision => {
                let value = read_string(&mut self.reader)?;
                // Validate that the string represents a valid number
                self.validate_high_precision_number(&value)?;
                Ok(UbjsonValue::HighPrecision(value))
            }
            UbjsonType::Char => {
                let value = read_char(&mut self.reader)?;
                Ok(UbjsonValue::Char(value))
            }
            UbjsonType::String => {
                let value = read_string(&mut self.reader)?;
                Ok(UbjsonValue::String(value))
            }
            UbjsonType::NoOp => {
                // Skip no-op markers and read the next value
                self.deserialize_value()
            }
            UbjsonType::ArrayStart => {
                self.deserialize_array()
            }
            UbjsonType::ObjectStart => {
                self.deserialize_object()
            }
            UbjsonType::ArrayEnd | UbjsonType::ObjectEnd => {
                Err(UbjsonError::invalid_format(format!(
                    "Unexpected container end marker: {}",
                    type_marker
                )))
            }
        }
    }

    /// Deserialize a standard array from the reader.
    fn deserialize_array(&mut self) -> Result<UbjsonValue> {
        // Increment depth and check limit
        self.current_depth += 1;
        if self.current_depth > self.max_depth {
            self.current_depth -= 1;
            return Err(UbjsonError::DepthLimitExceeded(self.max_depth));
        }

        let mut elements = Vec::new();
        let mut element_count = 0;

        // Read elements until we encounter the array end marker
        loop {
            // Check size limit before reading each element
            if element_count >= self.max_size {
                self.current_depth -= 1;
                return Err(UbjsonError::SizeLimitExceeded(self.max_size));
            }

            let type_marker = read_type_marker(&mut self.reader)?;
            
            if type_marker == UbjsonType::ArrayEnd {
                break;
            }

            // Deserialize the element with the known type marker
            let element = self.deserialize_value_with_type(type_marker)?;
            elements.push(element);
            element_count += 1;
        }

        self.current_depth -= 1;
        Ok(UbjsonValue::Array(elements))
    }

    /// Deserialize a standard object from the reader.
    fn deserialize_object(&mut self) -> Result<UbjsonValue> {
        // Increment depth and check limit
        self.current_depth += 1;
        if self.current_depth > self.max_depth {
            self.current_depth -= 1;
            return Err(UbjsonError::DepthLimitExceeded(self.max_depth));
        }

        let mut pairs = std::collections::HashMap::new();
        let mut pair_count = 0;

        // Read key-value pairs until we encounter the object end marker
        loop {
            // Check size limit before reading each pair
            if pair_count >= self.max_size {
                self.current_depth -= 1;
                return Err(UbjsonError::SizeLimitExceeded(self.max_size));
            }

            let type_marker = read_type_marker(&mut self.reader)?;
            
            if type_marker == UbjsonType::ObjectEnd {
                break;
            }

            // Keys must be strings in UBJSON objects
            if type_marker != UbjsonType::String {
                self.current_depth -= 1;
                return Err(UbjsonError::invalid_format(format!(
                    "Object keys must be strings, found: {}",
                    type_marker
                )));
            }

            // Read the key string
            let key = read_string(&mut self.reader)?;

            // Check for duplicate keys
            if pairs.contains_key(&key) {
                self.current_depth -= 1;
                return Err(UbjsonError::invalid_format(format!(
                    "Duplicate key in object: '{}'",
                    key
                )));
            }

            // Read the value
            let value = self.deserialize_value()?;
            pairs.insert(key, value);
            pair_count += 1;
        }

        self.current_depth -= 1;
        Ok(UbjsonValue::Object(pairs))
    }

    /// Validate that a high-precision number string is valid.
    fn validate_high_precision_number(&self, value: &str) -> Result<()> {
        if value.is_empty() {
            return Err(UbjsonError::InvalidHighPrecision(
                "Empty high-precision number".to_string()
            ));
        }

        // Basic validation - check if it looks like a number
        // Allow: digits, decimal point, scientific notation (e/E), signs
        let mut chars = value.chars().peekable();
        
        // Optional leading sign
        if let Some(&first) = chars.peek() {
            if first == '+' || first == '-' {
                chars.next();
            }
        }

        let mut has_digits = false;
        let mut has_decimal = false;
        let mut has_exponent = false;

        while let Some(ch) = chars.next() {
            match ch {
                '0'..='9' => {
                    has_digits = true;
                }
                '.' => {
                    if has_decimal || has_exponent {
                        return Err(UbjsonError::InvalidHighPrecision(
                            format!("Invalid decimal point in high-precision number: {}", value)
                        ));
                    }
                    has_decimal = true;
                }
                'e' | 'E' => {
                    if !has_digits || has_exponent {
                        return Err(UbjsonError::InvalidHighPrecision(
                            format!("Invalid exponent in high-precision number: {}", value)
                        ));
                    }
                    has_exponent = true;
                    
                    // Optional sign after exponent
                    if let Some(&next) = chars.peek() {
                        if next == '+' || next == '-' {
                            chars.next();
                        }
                    }
                }
                _ => {
                    return Err(UbjsonError::InvalidHighPrecision(
                        format!("Invalid character '{}' in high-precision number: {}", ch, value)
                    ));
                }
            }
        }

        if !has_digits {
            return Err(UbjsonError::InvalidHighPrecision(
                format!("No digits found in high-precision number: {}", value)
            ));
        }

        Ok(())
    }

    /// Get the current nesting depth.
    pub fn current_depth(&self) -> usize {
        self.current_depth
    }

    /// Get the maximum allowed nesting depth.
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Get the maximum allowed container size.
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_deserialize_null() {
        let data = vec![b'Z'];
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::Null);
    }

    #[test]
    fn test_deserialize_bool_true() {
        let data = vec![b'T'];
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::Bool(true));
    }

    #[test]
    fn test_deserialize_bool_false() {
        let data = vec![b'F'];
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::Bool(false));
    }

    #[test]
    fn test_deserialize_int8() {
        let data = vec![b'i', 0x7F]; // 127
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::Int8(127));
    }

    #[test]
    fn test_deserialize_int8_negative() {
        let data = vec![b'i', 0x80]; // -128
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::Int8(-128));
    }

    #[test]
    fn test_deserialize_uint8() {
        let data = vec![b'U', 0xFF]; // 255
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::UInt8(255));
    }

    #[test]
    fn test_deserialize_int16() {
        let data = vec![b'I', 0x7F, 0xFF]; // 32767
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::Int16(32767));
    }

    #[test]
    fn test_deserialize_int32() {
        let data = vec![b'l', 0x7F, 0xFF, 0xFF, 0xFF]; // 2147483647
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::Int32(2147483647));
    }

    #[test]
    fn test_deserialize_int64() {
        let data = vec![b'L', 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]; // 9223372036854775807
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::Int64(9223372036854775807));
    }

    #[test]
    fn test_deserialize_float32() {
        let data = vec![b'd', 0x40, 0x49, 0x0F, 0xDB]; // π as f32
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        if let UbjsonValue::Float32(value) = result {
            assert!((value - std::f32::consts::PI).abs() < 0.0001);
        } else {
            panic!("Expected Float32 value");
        }
    }

    #[test]
    fn test_deserialize_float64() {
        let data = vec![b'D', 0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x18]; // π as f64
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        if let UbjsonValue::Float64(value) = result {
            assert!((value - std::f64::consts::PI).abs() < 0.000000001);
        } else {
            panic!("Expected Float64 value");
        }
    }

    #[test]
    fn test_deserialize_char() {
        let data = vec![b'C', b'A'];
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::Char('A'));
    }

    #[test]
    fn test_deserialize_char_unicode() {
        // UTF-8 encoding of '€' (Euro symbol)
        let data = vec![b'C', 0xE2, 0x82, 0xAC];
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::Char('€'));
    }

    #[test]
    fn test_deserialize_string() {
        // String "Hello" with length prefix
        let data = vec![b'S', b'U', 5, b'H', b'e', b'l', b'l', b'o'];
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::String("Hello".to_string()));
    }

    #[test]
    fn test_deserialize_string_unicode() {
        // String "Hello 世界" with UTF-8 encoding
        let hello_world = "Hello 世界";
        let bytes = hello_world.as_bytes();
        let mut data = vec![b'S', b'U', bytes.len() as u8];
        data.extend_from_slice(bytes);
        
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::String(hello_world.to_string()));
    }

    #[test]
    fn test_deserialize_high_precision() {
        // High-precision number "3.141592653589793238462643383279"
        let number_str = "3.141592653589793238462643383279";
        let bytes = number_str.as_bytes();
        let mut data = vec![b'H', b'U', bytes.len() as u8];
        data.extend_from_slice(bytes);
        
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::HighPrecision(number_str.to_string()));
    }

    #[test]
    fn test_deserialize_high_precision_scientific() {
        // High-precision number in scientific notation
        let number_str = "1.23456789e-10";
        let bytes = number_str.as_bytes();
        let mut data = vec![b'H', b'U', bytes.len() as u8];
        data.extend_from_slice(bytes);
        
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::HighPrecision(number_str.to_string()));
    }

    #[test]
    fn test_deserialize_no_op() {
        // No-op followed by a null value
        let data = vec![b'N', b'Z'];
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::Null);
    }

    #[test]
    fn test_invalid_type_marker() {
        let data = vec![0xFF]; // Invalid type marker
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbjsonError::InvalidTypeMarker(0xFF)));
    }

    #[test]
    fn test_unexpected_eof() {
        let data = vec![b'i']; // Int8 marker but no data
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_utf8_string() {
        // Invalid UTF-8 sequence
        let data = vec![b'S', b'U', 2, 0xFF, 0xFE];
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbjsonError::InvalidUtf8(_)));
    }

    #[test]
    fn test_invalid_high_precision_empty() {
        let data = vec![b'H', b'U', 0]; // Empty high-precision number
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbjsonError::InvalidHighPrecision(_)));
    }

    #[test]
    fn test_invalid_high_precision_format() {
        // Invalid high-precision number with letters
        let number_str = "not_a_number";
        let bytes = number_str.as_bytes();
        let mut data = vec![b'H', b'U', bytes.len() as u8];
        data.extend_from_slice(bytes);
        
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbjsonError::InvalidHighPrecision(_)));
    }

    #[test]
    fn test_depth_limit() {
        let data = vec![b'Z']; // Simple null value
        let mut deserializer = UbjsonDeserializer::with_limits(Cursor::new(data), 0, 1000);
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbjsonError::DepthLimitExceeded(0)));
    }

    #[test]
    fn test_deserialize_empty_array() {
        let data = vec![b'[', b']']; // Empty array
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::Array(vec![]));
    }

    #[test]
    fn test_deserialize_array_with_primitives() {
        // Array with [null, true, 42]
        let data = vec![
            b'[',           // Array start
            b'Z',           // null
            b'T',           // true
            b'i', 42,       // int8(42)
            b']',           // Array end
        ];
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        
        let expected = UbjsonValue::Array(vec![
            UbjsonValue::Null,
            UbjsonValue::Bool(true),
            UbjsonValue::Int8(42),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_nested_arrays() {
        // Array with [[1, 2], [3]]
        let data = vec![
            b'[',           // Outer array start
            b'[',           // Inner array 1 start
            b'i', 1,        // int8(1)
            b'i', 2,        // int8(2)
            b']',           // Inner array 1 end
            b'[',           // Inner array 2 start
            b'i', 3,        // int8(3)
            b']',           // Inner array 2 end
            b']',           // Outer array end
        ];
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        
        let expected = UbjsonValue::Array(vec![
            UbjsonValue::Array(vec![
                UbjsonValue::Int8(1),
                UbjsonValue::Int8(2),
            ]),
            UbjsonValue::Array(vec![
                UbjsonValue::Int8(3),
            ]),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_empty_object() {
        let data = vec![b'{', b'}']; // Empty object
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        assert_eq!(result, UbjsonValue::Object(std::collections::HashMap::new()));
    }

    #[test]
    fn test_deserialize_object_with_primitives() {
        // Object with {"name": "John", "age": 30, "active": true}
        let mut data = vec![b'{']; // Object start
        
        // Key "name"
        data.push(b'S');
        data.push(b'U');
        data.push(4); // length
        data.extend_from_slice(b"name");
        // Value "John"
        data.push(b'S');
        data.push(b'U');
        data.push(4); // length
        data.extend_from_slice(b"John");
        
        // Key "age"
        data.push(b'S');
        data.push(b'U');
        data.push(3); // length
        data.extend_from_slice(b"age");
        // Value 30
        data.push(b'i');
        data.push(30);
        
        // Key "active"
        data.push(b'S');
        data.push(b'U');
        data.push(6); // length
        data.extend_from_slice(b"active");
        // Value true
        data.push(b'T');
        
        data.push(b'}'); // Object end
        
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        
        let mut expected_map = std::collections::HashMap::new();
        expected_map.insert("name".to_string(), UbjsonValue::String("John".to_string()));
        expected_map.insert("age".to_string(), UbjsonValue::Int8(30));
        expected_map.insert("active".to_string(), UbjsonValue::Bool(true));
        
        assert_eq!(result, UbjsonValue::Object(expected_map));
    }

    #[test]
    fn test_deserialize_nested_objects() {
        // Object with {"user": {"name": "John", "id": 1}}
        let mut data = vec![b'{']; // Outer object start
        
        // Key "user"
        data.push(b'S');
        data.push(b'U');
        data.push(4); // length
        data.extend_from_slice(b"user");
        
        // Value: nested object
        data.push(b'{'); // Inner object start
        
        // Key "name"
        data.push(b'S');
        data.push(b'U');
        data.push(4); // length
        data.extend_from_slice(b"name");
        // Value "John"
        data.push(b'S');
        data.push(b'U');
        data.push(4); // length
        data.extend_from_slice(b"John");
        
        // Key "id"
        data.push(b'S');
        data.push(b'U');
        data.push(2); // length
        data.extend_from_slice(b"id");
        // Value 1
        data.push(b'i');
        data.push(1);
        
        data.push(b'}'); // Inner object end
        data.push(b'}'); // Outer object end
        
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        
        let mut inner_map = std::collections::HashMap::new();
        inner_map.insert("name".to_string(), UbjsonValue::String("John".to_string()));
        inner_map.insert("id".to_string(), UbjsonValue::Int8(1));
        
        let mut outer_map = std::collections::HashMap::new();
        outer_map.insert("user".to_string(), UbjsonValue::Object(inner_map));
        
        assert_eq!(result, UbjsonValue::Object(outer_map));
    }

    #[test]
    fn test_deserialize_mixed_containers() {
        // Object with {"numbers": [1, 2, 3], "empty": []}
        let mut data = vec![b'{']; // Object start
        
        // Key "numbers"
        data.push(b'S');
        data.push(b'U');
        data.push(7); // length
        data.extend_from_slice(b"numbers");
        
        // Value: array [1, 2, 3]
        data.push(b'['); // Array start
        data.push(b'i'); data.push(1);
        data.push(b'i'); data.push(2);
        data.push(b'i'); data.push(3);
        data.push(b']'); // Array end
        
        // Key "empty"
        data.push(b'S');
        data.push(b'U');
        data.push(5); // length
        data.extend_from_slice(b"empty");
        
        // Value: empty array
        data.push(b'['); // Array start
        data.push(b']'); // Array end
        
        data.push(b'}'); // Object end
        
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value().unwrap();
        
        let mut expected_map = std::collections::HashMap::new();
        expected_map.insert("numbers".to_string(), UbjsonValue::Array(vec![
            UbjsonValue::Int8(1),
            UbjsonValue::Int8(2),
            UbjsonValue::Int8(3),
        ]));
        expected_map.insert("empty".to_string(), UbjsonValue::Array(vec![]));
        
        assert_eq!(result, UbjsonValue::Object(expected_map));
    }

    #[test]
    fn test_object_duplicate_key_error() {
        // Object with duplicate key "name"
        let mut data = vec![b'{']; // Object start
        
        // First "name" key
        data.push(b'S');
        data.push(b'U');
        data.push(4); // length
        data.extend_from_slice(b"name");
        data.push(b'S');
        data.push(b'U');
        data.push(4); // length
        data.extend_from_slice(b"John");
        
        // Second "name" key (duplicate)
        data.push(b'S');
        data.push(b'U');
        data.push(4); // length
        data.extend_from_slice(b"name");
        data.push(b'S');
        data.push(b'U');
        data.push(4); // length
        data.extend_from_slice(b"Jane");
        
        data.push(b'}'); // Object end
        
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbjsonError::InvalidFormat(_)));
    }

    #[test]
    fn test_object_non_string_key_error() {
        // Object with non-string key
        let data = vec![
            b'{',           // Object start
            b'i', 42,       // int8(42) as key (invalid)
            b'S', b'U', 5, b'v', b'a', b'l', b'u', b'e', // "value"
            b'}',           // Object end
        ];
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbjsonError::InvalidFormat(_)));
    }

    #[test]
    fn test_array_depth_limit() {
        // Create deeply nested array that exceeds depth limit
        let mut data = vec![];
        let depth = 5;
        
        // Create nested arrays: [[[[[null]]]]]
        for _ in 0..depth {
            data.push(b'[');
        }
        data.push(b'Z'); // null value
        for _ in 0..depth {
            data.push(b']');
        }
        
        let mut deserializer = UbjsonDeserializer::with_limits(Cursor::new(data), 3, 1000);
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbjsonError::DepthLimitExceeded(3)));
    }

    #[test]
    fn test_object_depth_limit() {
        // Create deeply nested object that exceeds depth limit
        let mut data = vec![];
        let depth = 5;
        
        // Create nested objects: {"a": {"b": {"c": {"d": {"e": null}}}}}
        for i in 0..depth {
            data.push(b'{');
            data.push(b'S');
            data.push(b'U');
            data.push(1); // key length
            data.push(b'a' + i as u8); // key: "a", "b", "c", etc.
        }
        data.push(b'Z'); // null value
        for _ in 0..depth {
            data.push(b'}');
        }
        
        let mut deserializer = UbjsonDeserializer::with_limits(Cursor::new(data), 3, 1000);
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbjsonError::DepthLimitExceeded(3)));
    }

    #[test]
    fn test_array_size_limit() {
        // Create array with too many elements
        let mut data = vec![b'[']; // Array start
        
        let size_limit = 3;
        for i in 0..size_limit + 1 {
            data.push(b'i');
            data.push(i as u8);
        }
        data.push(b']'); // Array end
        
        let mut deserializer = UbjsonDeserializer::with_limits(Cursor::new(data), 1000, size_limit);
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbjsonError::SizeLimitExceeded(_)));
    }

    #[test]
    fn test_object_size_limit() {
        // Create object with too many key-value pairs
        let mut data = vec![b'{']; // Object start
        
        let size_limit = 2;
        for i in 0..size_limit + 1 {
            // Key
            data.push(b'S');
            data.push(b'U');
            data.push(4); // length (corrected)
            data.extend_from_slice(b"key");
            data.push(b'0' + i as u8); // Make keys unique: "key0", "key1", etc.
            
            // Value
            data.push(b'i');
            data.push(i as u8);
        }
        data.push(b'}'); // Object end
        
        let mut deserializer = UbjsonDeserializer::with_limits(Cursor::new(data), 1000, size_limit);
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
        match result.unwrap_err() {
            UbjsonError::SizeLimitExceeded(_) => {}, // Expected
            other => panic!("Expected SizeLimitExceeded, got: {:?}", other),
        }
    }

    #[test]
    fn test_unexpected_container_end() {
        let data = vec![b']']; // Array end marker without start
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbjsonError::InvalidFormat(_)));
    }

    #[test]
    fn test_unexpected_object_end() {
        let data = vec![b'}']; // Object end marker without start
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbjsonError::InvalidFormat(_)));
    }
}