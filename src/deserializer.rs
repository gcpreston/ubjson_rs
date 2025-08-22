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
            UbjsonType::ArrayStart | UbjsonType::ObjectStart => {
                // Container deserialization will be implemented in later tasks
                Err(UbjsonError::unsupported_type(format!(
                    "Container deserialization not yet implemented: {}",
                    type_marker
                )))
            }
            UbjsonType::ArrayEnd | UbjsonType::ObjectEnd => {
                Err(UbjsonError::invalid_format(format!(
                    "Unexpected container end marker: {}",
                    type_marker
                )))
            }
        }
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
    fn test_container_not_implemented() {
        let data = vec![b'[']; // Array start marker
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbjsonError::UnsupportedType(_)));
    }

    #[test]
    fn test_unexpected_container_end() {
        let data = vec![b']']; // Array end marker without start
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbjsonError::InvalidFormat(_)));
    }
}