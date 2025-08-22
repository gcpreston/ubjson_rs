//! Tests for UBJSON container optimization features.

use ubjson_rs::{UbjsonSerializer, UbjsonValue, UbjsonType};
use std::collections::HashMap;

#[test]
fn test_homogeneous_int_array_optimization() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
    
    let array = UbjsonValue::Array(vec![
        UbjsonValue::Int32(100),
        UbjsonValue::Int32(200),
        UbjsonValue::Int32(300),
    ]);
    
    serializer.serialize_value(&array).unwrap();
    
    // Should be optimized: [$][l][#][count][values...]
    assert_eq!(buffer[0], b'[');  // Array start
    assert_eq!(buffer[1], b'$');  // Type marker
    assert_eq!(buffer[2], b'l');  // Int32 type
    assert_eq!(buffer[3], b'#');  // Count marker
    assert_eq!(buffer[4], b'U');  // Count type (uint8)
    assert_eq!(buffer[5], 3);     // Count value
    
    // Followed by 3 * 4 bytes for the Int32 values
    assert_eq!(buffer.len(), 6 + 12); // 6 bytes header + 12 bytes data
}

#[test]
fn test_homogeneous_string_object_optimization() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
    
    let mut map = HashMap::new();
    map.insert("name".to_string(), UbjsonValue::String("Alice".to_string()));
    map.insert("city".to_string(), UbjsonValue::String("Boston".to_string()));
    
    let object = UbjsonValue::Object(map);
    serializer.serialize_value(&object).unwrap();
    
    // Should be optimized: {$][S][#][count][key-value pairs...]
    assert_eq!(buffer[0], b'{');  // Object start
    assert_eq!(buffer[1], b'$');  // Type marker
    assert_eq!(buffer[2], b'S');  // String type
    assert_eq!(buffer[3], b'#');  // Count marker
    assert_eq!(buffer[4], b'U');  // Count type (uint8)
    assert_eq!(buffer[5], 2);     // Count value
    
    // Rest contains key-value pairs without value type markers
    assert!(buffer.len() > 6);
}

#[test]
fn test_mixed_type_array_no_optimization() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
    
    let array = UbjsonValue::Array(vec![
        UbjsonValue::Int32(42),
        UbjsonValue::String("hello".to_string()),
        UbjsonValue::Bool(true),
    ]);
    
    serializer.serialize_value(&array).unwrap();
    
    // Should use standard format
    assert_eq!(buffer[0], b'[');  // Array start
    assert_eq!(buffer[buffer.len() - 1], b']');  // Array end
    
    // Should not have optimization markers at the beginning
    assert_ne!(buffer[1], b'$');
}

#[test]
fn test_mixed_type_object_no_optimization() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
    
    let mut map = HashMap::new();
    map.insert("number".to_string(), UbjsonValue::Int32(42));
    map.insert("text".to_string(), UbjsonValue::String("hello".to_string()));
    
    let object = UbjsonValue::Object(map);
    serializer.serialize_value(&object).unwrap();
    
    // Should use standard format
    assert_eq!(buffer[0], b'{');  // Object start
    assert_eq!(buffer[buffer.len() - 1], b'}');  // Object end
    
    // Should not have optimization markers at the beginning
    assert_ne!(buffer[1], b'$');
}

#[test]
fn test_strongly_typed_array_with_count() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    
    let array = UbjsonValue::StronglyTypedArray {
        element_type: UbjsonType::UInt8,
        count: Some(4),
        elements: vec![
            UbjsonValue::UInt8(10),
            UbjsonValue::UInt8(20),
            UbjsonValue::UInt8(30),
            UbjsonValue::UInt8(40),
        ],
    };
    
    serializer.serialize_value(&array).unwrap();
    
    let expected = vec![
        b'[',           // Array start
        b'$',           // Type marker
        b'U',           // UInt8 type
        b'#',           // Count marker
        b'U', 4,        // Count (4 as uint8)
        10, 20, 30, 40, // Elements without type markers
    ];
    assert_eq!(buffer, expected);
}

#[test]
fn test_strongly_typed_array_without_count() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    
    let array = UbjsonValue::StronglyTypedArray {
        element_type: UbjsonType::Int16,
        count: None,
        elements: vec![
            UbjsonValue::Int16(1000),
            UbjsonValue::Int16(2000),
        ],
    };
    
    serializer.serialize_value(&array).unwrap();
    
    // Should have type marker but no count, and end with array end marker
    assert_eq!(buffer[0], b'[');  // Array start
    assert_eq!(buffer[1], b'$');  // Type marker
    assert_eq!(buffer[2], b'I');  // Int16 type
    assert_eq!(buffer[buffer.len() - 1], b']');  // Array end
    
    // Should not have count marker
    assert_ne!(buffer[3], b'#');
}

#[test]
fn test_strongly_typed_object_with_count() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    
    let mut pairs = HashMap::new();
    pairs.insert("x".to_string(), UbjsonValue::Float32(1.5));
    pairs.insert("y".to_string(), UbjsonValue::Float32(2.5));
    
    let object = UbjsonValue::StronglyTypedObject {
        value_type: UbjsonType::Float32,
        count: Some(2),
        pairs,
    };
    
    serializer.serialize_value(&object).unwrap();
    
    // Should have optimization markers
    assert_eq!(buffer[0], b'{');  // Object start
    assert_eq!(buffer[1], b'$');  // Type marker
    assert_eq!(buffer[2], b'd');  // Float32 type
    assert_eq!(buffer[3], b'#');  // Count marker
    assert_eq!(buffer[4], b'U');  // Count type
    assert_eq!(buffer[5], 2);     // Count value
    
    // Should not end with object end marker since count is provided
    assert_ne!(buffer[buffer.len() - 1], b'}');
}

#[test]
fn test_strongly_typed_object_without_count() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    
    let mut pairs = HashMap::new();
    pairs.insert("a".to_string(), UbjsonValue::Int64(1000000));
    pairs.insert("b".to_string(), UbjsonValue::Int64(2000000));
    
    let object = UbjsonValue::StronglyTypedObject {
        value_type: UbjsonType::Int64,
        count: None,
        pairs,
    };
    
    serializer.serialize_value(&object).unwrap();
    
    // Should have type marker but no count, and end with object end marker
    assert_eq!(buffer[0], b'{');  // Object start
    assert_eq!(buffer[1], b'$');  // Type marker
    assert_eq!(buffer[2], b'L');  // Int64 type
    assert_eq!(buffer[buffer.len() - 1], b'}');  // Object end
    
    // Should not have count marker immediately after type
    assert_ne!(buffer[3], b'#');
}

#[test]
fn test_optimization_disabled_for_homogeneous_array() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, false);
    
    let array = UbjsonValue::Array(vec![
        UbjsonValue::Bool(true),
        UbjsonValue::Bool(false),
        UbjsonValue::Bool(true),
    ]);
    
    serializer.serialize_value(&array).unwrap();
    
    // Should use standard format even though array is homogeneous
    let expected = vec![
        b'[',           // Array start
        b'T',           // True
        b'F',           // False
        b'T',           // True
        b']',           // Array end
    ];
    assert_eq!(buffer, expected);
}

#[test]
fn test_optimization_disabled_for_homogeneous_object() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, false);
    
    let mut map = HashMap::new();
    map.insert("a".to_string(), UbjsonValue::Char('x'));
    map.insert("b".to_string(), UbjsonValue::Char('y'));
    
    let object = UbjsonValue::Object(map);
    serializer.serialize_value(&object).unwrap();
    
    // Should use standard format
    assert_eq!(buffer[0], b'{');  // Object start
    assert_eq!(buffer[buffer.len() - 1], b'}');  // Object end
    
    // Should not have optimization markers
    assert_ne!(buffer[1], b'$');
}

#[test]
fn test_nested_containers_optimization() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
    
    // Outer array contains objects, so it won't be optimized
    // But the inner objects are homogeneous, so they can be optimized
    let mut obj1 = HashMap::new();
    obj1.insert("a".to_string(), UbjsonValue::Int8(1));
    obj1.insert("b".to_string(), UbjsonValue::Int8(2));
    
    let mut obj2 = HashMap::new();
    obj2.insert("c".to_string(), UbjsonValue::Int8(3));
    obj2.insert("d".to_string(), UbjsonValue::Int8(4));
    
    let array = UbjsonValue::Array(vec![
        UbjsonValue::Object(obj1),
        UbjsonValue::Object(obj2),
    ]);
    
    serializer.serialize_value(&array).unwrap();
    
    // Outer array should not be optimized (contains containers)
    assert_eq!(buffer[0], b'[');  // Array start
    assert_eq!(buffer[buffer.len() - 1], b']');  // Array end
    assert_ne!(buffer[1], b'$');  // No optimization for outer array
    
    // But inner objects should be optimized
    // Find the first object start after the array start
    let first_obj_pos = buffer.iter().position(|&b| b == b'{').unwrap();
    assert_eq!(buffer[first_obj_pos + 1], b'$');  // Type marker for first object
}

#[test]
fn test_empty_containers_no_optimization() {
    // Test empty array
    {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
        
        let empty_array = UbjsonValue::Array(vec![]);
        serializer.serialize_value(&empty_array).unwrap();
        assert_eq!(buffer, vec![b'[', b']']);
    }
    
    // Test empty object
    {
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
        
        let empty_object = UbjsonValue::Object(HashMap::new());
        serializer.serialize_value(&empty_object).unwrap();
        assert_eq!(buffer, vec![b'{', b'}']);
    }
}

#[test]
fn test_single_element_array_optimization() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
    
    let array = UbjsonValue::Array(vec![UbjsonValue::Float64(3.14159)]);
    serializer.serialize_value(&array).unwrap();
    
    // Single element arrays should still be optimized
    assert_eq!(buffer[0], b'[');  // Array start
    assert_eq!(buffer[1], b'$');  // Type marker
    assert_eq!(buffer[2], b'D');  // Float64 type
    assert_eq!(buffer[3], b'#');  // Count marker
    assert_eq!(buffer[4], b'U');  // Count type
    assert_eq!(buffer[5], 1);     // Count value
    
    // Followed by 8 bytes for the Float64 value
    assert_eq!(buffer.len(), 6 + 8);
}

#[test]
fn test_large_count_optimization() {
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
    
    // Create an array with more than 255 elements to test larger count encoding
    let elements: Vec<UbjsonValue> = (0..300).map(|i| UbjsonValue::UInt8(i as u8)).collect();
    let array = UbjsonValue::Array(elements);
    
    serializer.serialize_value(&array).unwrap();
    
    assert_eq!(buffer[0], b'[');  // Array start
    assert_eq!(buffer[1], b'$');  // Type marker
    assert_eq!(buffer[2], b'U');  // UInt8 type
    assert_eq!(buffer[3], b'#');  // Count marker
    assert_eq!(buffer[4], b'I');  // Count type (Int16 since 300 > 255)
    
    // Count value (300 as big-endian Int16)
    let count_bytes = 300i16.to_be_bytes();
    assert_eq!(buffer[5], count_bytes[0]);
    assert_eq!(buffer[6], count_bytes[1]);
    
    // Followed by 300 bytes for the UInt8 values
    assert_eq!(buffer.len(), 7 + 300);
}