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

#[test]
fn test_container_serialization_integration() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);

    // Test empty containers
    serializer.serialize_value(&UbjsonValue::Array(vec![])).unwrap();
    serializer.serialize_value(&UbjsonValue::Object(std::collections::HashMap::new())).unwrap();

    // Test array with mixed types
    let mixed_array = UbjsonValue::Array(vec![
        UbjsonValue::Null,
        UbjsonValue::Bool(true),
        UbjsonValue::Int32(42),
        UbjsonValue::String("hello".to_string()),
    ]);
    serializer.serialize_value(&mixed_array).unwrap();

    // Test object with mixed types
    let mut mixed_object = std::collections::HashMap::new();
    mixed_object.insert("null_val".to_string(), UbjsonValue::Null);
    mixed_object.insert("bool_val".to_string(), UbjsonValue::Bool(false));
    mixed_object.insert("int_val".to_string(), UbjsonValue::Int16(1000));
    mixed_object.insert("str_val".to_string(), UbjsonValue::String("world".to_string()));
    
    serializer.serialize_value(&UbjsonValue::Object(mixed_object)).unwrap();

    // Verify that we have written data
    assert!(!buffer.is_empty());
    
    // Check for expected markers
    let array_starts = buffer.iter().filter(|&&b| b == b'[').count();
    let array_ends = buffer.iter().filter(|&&b| b == b']').count();
    let object_starts = buffer.iter().filter(|&&b| b == b'{').count();
    let object_ends = buffer.iter().filter(|&&b| b == b'}').count();
    
    assert_eq!(array_starts, 2); // Empty array + mixed array
    assert_eq!(array_ends, 2);
    assert_eq!(object_starts, 2); // Empty object + mixed object
    assert_eq!(object_ends, 2);
}

#[test]
fn test_deeply_nested_containers() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);

    // Create a complex nested structure
    let mut inner_object = std::collections::HashMap::new();
    inner_object.insert("level".to_string(), UbjsonValue::Int8(3));
    inner_object.insert("data".to_string(), UbjsonValue::Array(vec![
        UbjsonValue::String("nested".to_string()),
        UbjsonValue::Bool(true),
    ]));

    let middle_array = UbjsonValue::Array(vec![
        UbjsonValue::Object(inner_object),
        UbjsonValue::Int32(100),
    ]);

    let mut outer_object = std::collections::HashMap::new();
    outer_object.insert("nested_array".to_string(), middle_array);
    outer_object.insert("simple".to_string(), UbjsonValue::String("value".to_string()));

    let root = UbjsonValue::Array(vec![
        UbjsonValue::Object(outer_object),
        UbjsonValue::Null,
    ]);

    let result = serializer.serialize_value(&root);
    assert!(result.is_ok(), "Failed to serialize deeply nested structure");

    // Verify structure markers are balanced
    let array_starts = buffer.iter().filter(|&&b| b == b'[').count();
    let array_ends = buffer.iter().filter(|&&b| b == b']').count();
    let object_starts = buffer.iter().filter(|&&b| b == b'{').count();
    let object_ends = buffer.iter().filter(|&&b| b == b'}').count();
    
    assert_eq!(array_starts, array_ends);
    assert_eq!(object_starts, object_ends);
    assert!(array_starts > 0);
    assert!(object_starts > 0);
}

#[test]
fn test_large_containers() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);

    // Test large array
    let large_array: Vec<UbjsonValue> = (0..1000)
        .map(|i| UbjsonValue::Int32(i))
        .collect();
    
    let result = serializer.serialize_value(&UbjsonValue::Array(large_array));
    assert!(result.is_ok(), "Failed to serialize large array");

    // Test large object
    let mut large_object = std::collections::HashMap::new();
    for i in 0..500 {
        large_object.insert(format!("key_{}", i), UbjsonValue::Int32(i));
    }
    
    let result = serializer.serialize_value(&UbjsonValue::Object(large_object));
    assert!(result.is_ok(), "Failed to serialize large object");

    // Verify we have substantial data
    assert!(buffer.len() > 10000, "Buffer should be substantial for large containers");
}

#[test]
fn test_depth_limit_enforcement() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::with_depth_limit(&mut buffer, 3);

    // Create a structure that exceeds the depth limit
    let level4 = UbjsonValue::Array(vec![UbjsonValue::Int8(1)]);
    let level3 = UbjsonValue::Array(vec![level4]);
    let level2 = UbjsonValue::Array(vec![level3]);
    let level1 = UbjsonValue::Array(vec![level2]);

    let result = serializer.serialize_value(&level1);
    assert!(result.is_err(), "Should fail due to depth limit");
    
    match result {
        Err(ubjson_rs::UbjsonError::DepthLimitExceeded(depth)) => {
            assert_eq!(depth, 3);
        }
        _ => panic!("Expected DepthLimitExceeded error"),
    }
}

#[test]
fn test_container_serialization_with_optimization_flag() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);

    // Test that optimization flag doesn't break standard container serialization
    let array = UbjsonValue::Array(vec![
        UbjsonValue::Int8(1),
        UbjsonValue::Int8(2),
        UbjsonValue::Int8(3),
    ]);
    
    let result = serializer.serialize_value(&array);
    assert!(result.is_ok(), "Serialization should succeed with optimization flag");

    // Verify basic structure (optimization for standard containers will be implemented later)
    assert_eq!(buffer[0], b'[');
    assert_eq!(buffer[buffer.len() - 1], b']');
}

#[test]
fn test_object_key_serialization() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);

    // Test object with various key types
    let mut object = std::collections::HashMap::new();
    object.insert("".to_string(), UbjsonValue::Null); // Empty key
    object.insert("simple".to_string(), UbjsonValue::Bool(true));
    object.insert("with spaces".to_string(), UbjsonValue::Int8(42));
    object.insert("unicode_key_世界".to_string(), UbjsonValue::String("unicode_value".to_string()));
    object.insert("special!@#$%^&*()".to_string(), UbjsonValue::Float32(3.14));

    let result = serializer.serialize_value(&UbjsonValue::Object(object));
    assert!(result.is_ok(), "Should handle various key types");

    // Verify object structure
    assert_eq!(buffer[0], b'{');
    assert_eq!(buffer[buffer.len() - 1], b'}');
}