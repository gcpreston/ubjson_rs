use std::io::Cursor;
use ubjson_rs::encoding::*;
use ubjson_rs::{UbjsonError, UbjsonType};

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