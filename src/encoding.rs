//! Low-level binary encoding utilities for UBJSON format.
//!
//! This module provides functions for reading and writing UBJSON type markers,
//! length encoding/decoding, and integer encoding in big-endian format.

use std::io::{Read, Write};
use crate::error::{UbjsonError, Result};
use crate::types::UbjsonType;

/// Read a single byte from the reader and interpret it as a UBJSON type marker.
pub fn read_type_marker<R: Read>(reader: &mut R) -> Result<UbjsonType> {
    let mut buffer = [0u8; 1];
    reader.read_exact(&mut buffer)?;
    UbjsonType::from_byte(buffer[0])
}

/// Write a UBJSON type marker to the writer.
pub fn write_type_marker<W: Write>(writer: &mut W, type_marker: UbjsonType) -> Result<()> {
    writer.write_all(&[type_marker.to_byte()])?;
    Ok(())
}

/// Read a length value from the reader. Length is encoded as a UBJSON integer.
/// Returns the length as usize for container operations.
pub fn read_length<R: Read>(reader: &mut R) -> Result<usize> {
    let type_marker = read_type_marker(reader)?;
    
    match type_marker {
        UbjsonType::UInt8 => {
            let value = read_uint8(reader)?;
            Ok(value as usize)
        }
        UbjsonType::Int8 => {
            let value = read_int8(reader)?;
            if value < 0 {
                return Err(UbjsonError::invalid_format("Negative length not allowed"));
            }
            Ok(value as usize)
        }
        UbjsonType::Int16 => {
            let value = read_int16(reader)?;
            if value < 0 {
                return Err(UbjsonError::invalid_format("Negative length not allowed"));
            }
            Ok(value as usize)
        }
        UbjsonType::Int32 => {
            let value = read_int32(reader)?;
            if value < 0 {
                return Err(UbjsonError::invalid_format("Negative length not allowed"));
            }
            Ok(value as usize)
        }
        UbjsonType::Int64 => {
            let value = read_int64(reader)?;
            if value < 0 {
                return Err(UbjsonError::invalid_format("Negative length not allowed"));
            }
            if value > usize::MAX as i64 {
                return Err(UbjsonError::invalid_format("Length too large for platform"));
            }
            Ok(value as usize)
        }
        _ => Err(UbjsonError::invalid_format(format!(
            "Invalid length type marker: {}",
            type_marker
        ))),
    }
}

/// Write a length value to the writer using the most compact integer representation.
pub fn write_length<W: Write>(writer: &mut W, length: usize) -> Result<()> {
    if length <= u8::MAX as usize {
        write_type_marker(writer, UbjsonType::UInt8)?;
        write_uint8(writer, length as u8)?;
    } else if length <= i16::MAX as usize {
        write_type_marker(writer, UbjsonType::Int16)?;
        write_int16(writer, length as i16)?;
    } else if length <= i32::MAX as usize {
        write_type_marker(writer, UbjsonType::Int32)?;
        write_int32(writer, length as i32)?;
    } else {
        write_type_marker(writer, UbjsonType::Int64)?;
        write_int64(writer, length as i64)?;
    }
    Ok(())
}

/// Read a signed 8-bit integer from the reader.
pub fn read_int8<R: Read>(reader: &mut R) -> Result<i8> {
    let mut buffer = [0u8; 1];
    reader.read_exact(&mut buffer)?;
    Ok(i8::from_be_bytes(buffer))
}

/// Write a signed 8-bit integer to the writer.
pub fn write_int8<W: Write>(writer: &mut W, value: i8) -> Result<()> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

/// Read an unsigned 8-bit integer from the reader.
pub fn read_uint8<R: Read>(reader: &mut R) -> Result<u8> {
    let mut buffer = [0u8; 1];
    reader.read_exact(&mut buffer)?;
    Ok(u8::from_be_bytes(buffer))
}

/// Write an unsigned 8-bit integer to the writer.
pub fn write_uint8<W: Write>(writer: &mut W, value: u8) -> Result<()> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

/// Read a signed 16-bit integer from the reader in big-endian format.
pub fn read_int16<R: Read>(reader: &mut R) -> Result<i16> {
    let mut buffer = [0u8; 2];
    reader.read_exact(&mut buffer)?;
    Ok(i16::from_be_bytes(buffer))
}

/// Write a signed 16-bit integer to the writer in big-endian format.
pub fn write_int16<W: Write>(writer: &mut W, value: i16) -> Result<()> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

/// Read a signed 32-bit integer from the reader in big-endian format.
pub fn read_int32<R: Read>(reader: &mut R) -> Result<i32> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)?;
    Ok(i32::from_be_bytes(buffer))
}

/// Write a signed 32-bit integer to the writer in big-endian format.
pub fn write_int32<W: Write>(writer: &mut W, value: i32) -> Result<()> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

/// Read a signed 64-bit integer from the reader in big-endian format.
pub fn read_int64<R: Read>(reader: &mut R) -> Result<i64> {
    let mut buffer = [0u8; 8];
    reader.read_exact(&mut buffer)?;
    Ok(i64::from_be_bytes(buffer))
}

/// Write a signed 64-bit integer to the writer in big-endian format.
pub fn write_int64<W: Write>(writer: &mut W, value: i64) -> Result<()> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

/// Read a 32-bit floating-point number from the reader in big-endian format.
pub fn read_float32<R: Read>(reader: &mut R) -> Result<f32> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)?;
    Ok(f32::from_be_bytes(buffer))
}

/// Write a 32-bit floating-point number to the writer in big-endian format.
pub fn write_float32<W: Write>(writer: &mut W, value: f32) -> Result<()> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

/// Read a 64-bit floating-point number from the reader in big-endian format.
pub fn read_float64<R: Read>(reader: &mut R) -> Result<f64> {
    let mut buffer = [0u8; 8];
    reader.read_exact(&mut buffer)?;
    Ok(f64::from_be_bytes(buffer))
}

/// Write a 64-bit floating-point number to the writer in big-endian format.
pub fn write_float64<W: Write>(writer: &mut W, value: f64) -> Result<()> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

/// Read a UTF-8 string from the reader. The string is prefixed with its length.
pub fn read_string<R: Read>(reader: &mut R) -> Result<String> {
    let length = read_length(reader)?;
    let mut buffer = vec![0u8; length];
    reader.read_exact(&mut buffer)?;
    
    let string = std::str::from_utf8(&buffer)?;
    Ok(string.to_string())
}

/// Write a UTF-8 string to the writer, prefixed with its length.
pub fn write_string<W: Write>(writer: &mut W, value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    write_length(writer, bytes.len())?;
    writer.write_all(bytes)?;
    Ok(())
}

/// Read a single UTF-8 character from the reader.
pub fn read_char<R: Read>(reader: &mut R) -> Result<char> {
    let mut buffer = [0u8; 1];
    reader.read_exact(&mut buffer)?;
    
    // Handle multi-byte UTF-8 characters
    let first_byte = buffer[0];
    let char_len = if first_byte < 0x80 {
        1 // ASCII
    } else if first_byte < 0xE0 {
        2 // 2-byte UTF-8
    } else if first_byte < 0xF0 {
        3 // 3-byte UTF-8
    } else {
        4 // 4-byte UTF-8
    };
    
    if char_len > 1 {
        let mut full_buffer = vec![first_byte];
        let mut remaining = vec![0u8; char_len - 1];
        reader.read_exact(&mut remaining)?;
        full_buffer.extend_from_slice(&remaining);
        
        let string = std::str::from_utf8(&full_buffer)?;
        let chars: Vec<char> = string.chars().collect();
        if chars.len() != 1 {
            return Err(UbjsonError::InvalidChar(format!(
                "Expected single character, got {} characters",
                chars.len()
            )));
        }
        Ok(chars[0])
    } else {
        Ok(first_byte as char)
    }
}

/// Write a single UTF-8 character to the writer.
pub fn write_char<W: Write>(writer: &mut W, value: char) -> Result<()> {
    let mut buffer = [0u8; 4];
    let encoded = value.encode_utf8(&mut buffer);
    writer.write_all(encoded.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_type_marker_roundtrip() {
        let mut buffer = Vec::new();
        
        // Test various type markers
        let markers = [
            UbjsonType::Null,
            UbjsonType::True,
            UbjsonType::False,
            UbjsonType::Int32,
            UbjsonType::String,
            UbjsonType::ArrayStart,
            UbjsonType::ObjectStart,
        ];
        
        for &marker in &markers {
            buffer.clear();
            write_type_marker(&mut buffer, marker).unwrap();
            
            let mut cursor = Cursor::new(&buffer);
            let read_marker = read_type_marker(&mut cursor).unwrap();
            assert_eq!(marker, read_marker);
        }
    }

    #[test]
    fn test_integer_roundtrip() {
        let mut buffer = Vec::new();
        
        // Test int8
        buffer.clear();
        write_int8(&mut buffer, -42).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(read_int8(&mut cursor).unwrap(), -42);
        
        // Test uint8
        buffer.clear();
        write_uint8(&mut buffer, 255).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(read_uint8(&mut cursor).unwrap(), 255);
        
        // Test int16
        buffer.clear();
        write_int16(&mut buffer, -1000).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(read_int16(&mut cursor).unwrap(), -1000);
        
        // Test int32
        buffer.clear();
        write_int32(&mut buffer, -100000).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(read_int32(&mut cursor).unwrap(), -100000);
        
        // Test int64
        buffer.clear();
        write_int64(&mut buffer, -1000000000000i64).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(read_int64(&mut cursor).unwrap(), -1000000000000i64);
    }

    #[test]
    fn test_float_roundtrip() {
        let mut buffer = Vec::new();
        
        // Test float32
        buffer.clear();
        write_float32(&mut buffer, 3.14159).unwrap();
        let mut cursor = Cursor::new(&buffer);
        let read_value = read_float32(&mut cursor).unwrap();
        assert!((read_value - 3.14159).abs() < f32::EPSILON);
        
        // Test float64
        buffer.clear();
        write_float64(&mut buffer, 2.718281828459045).unwrap();
        let mut cursor = Cursor::new(&buffer);
        let read_value = read_float64(&mut cursor).unwrap();
        assert!((read_value - 2.718281828459045).abs() < f64::EPSILON);
    }

    #[test]
    fn test_length_encoding() {
        let mut buffer = Vec::new();
        
        // Test small length (fits in uint8)
        buffer.clear();
        write_length(&mut buffer, 42).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(read_length(&mut cursor).unwrap(), 42);
        
        // Test medium length (requires int16)
        buffer.clear();
        write_length(&mut buffer, 1000).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(read_length(&mut cursor).unwrap(), 1000);
        
        // Test large length (requires int32)
        buffer.clear();
        write_length(&mut buffer, 100000).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(read_length(&mut cursor).unwrap(), 100000);
    }

    #[test]
    fn test_string_roundtrip() {
        let mut buffer = Vec::new();
        
        let test_strings = [
            "",
            "Hello, World!",
            "UTF-8: 🦀 Rust",
            "Multi\nline\tstring",
        ];
        
        for test_string in &test_strings {
            buffer.clear();
            write_string(&mut buffer, test_string).unwrap();
            
            let mut cursor = Cursor::new(&buffer);
            let read_string = read_string(&mut cursor).unwrap();
            assert_eq!(&read_string, test_string);
        }
    }

    #[test]
    fn test_char_roundtrip() {
        let mut buffer = Vec::new();
        
        let test_chars = ['A', '🦀', '中', '\n', '\0'];
        
        for &test_char in &test_chars {
            buffer.clear();
            write_char(&mut buffer, test_char).unwrap();
            
            let mut cursor = Cursor::new(&buffer);
            let read_char = read_char(&mut cursor).unwrap();
            assert_eq!(read_char, test_char);
        }
    }

    #[test]
    fn test_big_endian_encoding() {
        let mut buffer = Vec::new();
        
        // Test that integers are encoded in big-endian format
        write_int32(&mut buffer, 0x12345678).unwrap();
        assert_eq!(buffer, vec![0x12, 0x34, 0x56, 0x78]);
        
        buffer.clear();
        write_int16(&mut buffer, 0x1234).unwrap();
        assert_eq!(buffer, vec![0x12, 0x34]);
    }

    #[test]
    fn test_invalid_type_marker() {
        let buffer = vec![0xFF]; // Invalid type marker
        let mut cursor = Cursor::new(&buffer);
        
        let result = read_type_marker(&mut cursor);
        assert!(result.is_err());
        match result.unwrap_err() {
            UbjsonError::InvalidTypeMarker(0xFF) => (),
            _ => panic!("Expected InvalidTypeMarker error"),
        }
    }

    #[test]
    fn test_negative_length_error() {
        let mut buffer = Vec::new();
        
        // Write a negative int8 as length
        write_type_marker(&mut buffer, UbjsonType::Int8).unwrap();
        write_int8(&mut buffer, -1).unwrap();
        
        let mut cursor = Cursor::new(&buffer);
        let result = read_length(&mut cursor);
        assert!(result.is_err());
        match result.unwrap_err() {
            UbjsonError::InvalidFormat(msg) => {
                assert!(msg.contains("Negative length not allowed"));
            }
            _ => panic!("Expected InvalidFormat error for negative length"),
        }
    }

    #[test]
    fn test_unexpected_eof() {
        let buffer = vec![]; // Empty buffer
        let mut cursor = Cursor::new(&buffer);
        
        let result = read_type_marker(&mut cursor);
        assert!(result.is_err());
        match result.unwrap_err() {
            UbjsonError::Io(_) => (), // Should be UnexpectedEof from io::Error
            _ => panic!("Expected IO error for unexpected EOF"),
        }
    }

    #[test]
    fn test_invalid_utf8_in_string() {
        let mut buffer = Vec::new();
        
        // Write length and invalid UTF-8 bytes
        write_length(&mut buffer, 4).unwrap();
        buffer.extend_from_slice(&[0xFF, 0xFE, 0xFD, 0xFC]); // Invalid UTF-8
        
        let mut cursor = Cursor::new(&buffer);
        let result = read_string(&mut cursor);
        assert!(result.is_err());
        match result.unwrap_err() {
            UbjsonError::InvalidUtf8(_) => (),
            _ => panic!("Expected InvalidUtf8 error"),
        }
    }
}