use ubjson_rs::{UbjsonSerializer, UbjsonValue};
use std::io::Cursor;

#[test]
fn test_basic_serialization_integration() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);

    // Test serializing various primitive types
    let values = vec![
        UbjsonValue::Null,
        UbjsonValue::Bool(true),
        UbjsonValue::Bool(false),
        UbjsonValue::Int8(-42),
        UbjsonValue::UInt8(255),
        UbjsonValue::Int16(1000),
        UbjsonValue::Int32(100000),
        UbjsonValue::Int64(1000000000000),
        UbjsonValue::Float32(3.14159),
        UbjsonValue::Float64(3.141592653589793),
        UbjsonValue::HighPrecision("3.141592653589793238462643383279502884197".to_string()),
        UbjsonValue::Char('A'),
        UbjsonValue::Char('π'),
        UbjsonValue::String("Hello, World!".to_string()),
        UbjsonValue::String("Hello, 世界!".to_string()),
    ];

    for value in &values {
        let result = serializer.serialize_value(value);
        assert!(result.is_ok(), "Failed to serialize value: {:?}", value);
    }

    // Verify that we have written some data
    assert!(!buffer.is_empty(), "Buffer should not be empty after serialization");
}

#[test]
fn test_serializer_with_cursor() {
    let cursor = Cursor::new(Vec::new());
    let mut serializer = UbjsonSerializer::new(cursor);

    serializer.serialize_value(&UbjsonValue::String("test".to_string())).unwrap();

    let cursor = serializer.into_writer();
    let buffer = cursor.into_inner();
    
    // Should contain: 'S' (string marker) + 'U' (uint8 length marker) + 4 (length) + "test"
    assert_eq!(buffer.len(), 7);
    assert_eq!(buffer[0], b'S');
    assert_eq!(buffer[1], b'U');
    assert_eq!(buffer[2], 4);
    assert_eq!(&buffer[3..], b"test");
}