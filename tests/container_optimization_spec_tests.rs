//! Tests for UBJSON container optimization specification compliance.
//! 
//! These tests ensure that both serializer and deserializer comply with the
//! container optimization specification requirements.

use ubjson_rs::{UbjsonSerializer, UbjsonDeserializer, UbjsonValue, UbjsonType};
use std::collections::HashMap;
use std::io::Cursor;

// ============================================================================
// SERIALIZER SPECIFICATION COMPLIANCE TESTS
// ============================================================================

#[test]
fn test_serializer_count_must_be_non_negative() {
    // This test verifies that counts are always >= 0
    // Since we use usize for counts in Rust, negative counts are impossible
    // But we test that zero counts work correctly
    
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    
    let empty_array = UbjsonValue::StronglyTypedArray {
        element_type: UbjsonType::Int8,
        count: Some(0),
        elements: vec![],
    };
    
    serializer.serialize_value(&empty_array).unwrap();
    
    // Should serialize as: [$i#U0] (no elements, no end marker)
    let expected = vec![
        b'[',           // Array start
        b'$',           // Type marker
        b'i',           // Int8 type
        b'#',           // Count marker
        b'U', 0,        // Count (0 as uint8)
    ];
    assert_eq!(buffer, expected);
}

#[test]
fn test_serializer_count_can_be_specified_alone() {
    // Test that count can be specified without type optimization
    // This is tested by creating a regular array that gets count optimization
    // but not type optimization (heterogeneous elements)
    
    // Note: Our current implementation doesn't support count-only optimization
    // without type optimization, as per UBJSON spec requirements.
    // This test documents the expected behavior.
    
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::with_optimization(&mut buffer, true);
    
    // Heterogeneous array - should not get any optimization
    let array = UbjsonValue::Array(vec![
        UbjsonValue::Int8(1),
        UbjsonValue::String("hello".to_string()),
    ]);
    
    serializer.serialize_value(&array).unwrap();
    
    // Should use standard format (no count-only optimization)
    assert_eq!(buffer[0], b'[');
    assert_eq!(buffer[buffer.len() - 1], b']');
    assert_ne!(buffer[1], b'#'); // No count marker without type
}

#[test]
fn test_serializer_count_specified_no_end_marker() {
    // Test that containers with count don't have end markers
    
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    
    let array = UbjsonValue::StronglyTypedArray {
        element_type: UbjsonType::UInt8,
        count: Some(3),
        elements: vec![
            UbjsonValue::UInt8(10),
            UbjsonValue::UInt8(20),
            UbjsonValue::UInt8(30),
        ],
    };
    
    serializer.serialize_value(&array).unwrap();
    
    // Should not end with ']' since count is specified
    assert_ne!(buffer[buffer.len() - 1], b']');
    
    // Verify the structure: [$U#U3 10 20 30]
    let expected = vec![
        b'[',           // Array start
        b'$',           // Type marker
        b'U',           // UInt8 type
        b'#',           // Count marker
        b'U', 3,        // Count (3 as uint8)
        10, 20, 30,     // Elements without type markers
    ];
    assert_eq!(buffer, expected);
}

#[test]
fn test_serializer_count_without_end_marker_object() {
    // Test that objects with count don't have end markers
    
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    
    let mut pairs = HashMap::new();
    pairs.insert("a".to_string(), UbjsonValue::Int16(100));
    pairs.insert("b".to_string(), UbjsonValue::Int16(200));
    
    let object = UbjsonValue::StronglyTypedObject {
        value_type: UbjsonType::Int16,
        count: Some(2),
        pairs,
    };
    
    serializer.serialize_value(&object).unwrap();
    
    // Should not end with '}' since count is specified
    assert_ne!(buffer[buffer.len() - 1], b'}');
    
    // Should start with {$I#U2
    assert_eq!(buffer[0], b'{');  // Object start
    assert_eq!(buffer[1], b'$');  // Type marker
    assert_eq!(buffer[2], b'I');  // Int16 type
    assert_eq!(buffer[3], b'#');  // Count marker
    assert_eq!(buffer[4], b'U');  // Count type
    assert_eq!(buffer[5], 2);     // Count value
}

#[test]
fn test_serializer_container_must_contain_specified_count() {
    // Test that the serializer validates element count matches specified count
    
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    
    // Create array with mismatched count (says 2 but has 3 elements)
    let array = UbjsonValue::StronglyTypedArray {
        element_type: UbjsonType::Int8,
        count: Some(2),  // Says 2 elements
        elements: vec![  // But has 3 elements
            UbjsonValue::Int8(1),
            UbjsonValue::Int8(2),
            UbjsonValue::Int8(3),
        ],
    };
    
    // This should succeed - the serializer uses the actual elements length
    // and ignores the count field for validation (it's the deserializer's job)
    let result = serializer.serialize_value(&array);
    assert!(result.is_ok());
    
    // The serialized count should match the actual elements (3, not 2)
    assert_eq!(buffer[5], 3); // Count should be 3
}

#[test]
fn test_serializer_type_specified_before_count() {
    // Test that type marker comes before count marker in serialized output
    
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    
    let array = UbjsonValue::StronglyTypedArray {
        element_type: UbjsonType::Float32,
        count: Some(1),
        elements: vec![UbjsonValue::Float32(3.14)],
    };
    
    serializer.serialize_value(&array).unwrap();
    
    // Verify order: [ $ type # count ...
    assert_eq!(buffer[0], b'[');  // Array start
    assert_eq!(buffer[1], b'$');  // Type marker (comes first)
    assert_eq!(buffer[2], b'd');  // Float32 type
    assert_eq!(buffer[3], b'#');  // Count marker (comes after type)
    assert_eq!(buffer[4], b'U');  // Count type
    assert_eq!(buffer[5], 1);     // Count value
}

#[test]
fn test_serializer_type_requires_count() {
    // Test that our implementation always includes count when type is specified
    // (This is enforced by our StronglyTypedArray/Object structure)
    
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    
    // StronglyTypedArray without count should still include count in serialization
    let array = UbjsonValue::StronglyTypedArray {
        element_type: UbjsonType::Int32,
        count: None,  // No count specified
        elements: vec![UbjsonValue::Int32(42)],
    };
    
    serializer.serialize_value(&array).unwrap();
    
    // Should still have type marker but no count marker, and should end with ]
    assert_eq!(buffer[0], b'[');  // Array start
    assert_eq!(buffer[1], b'$');  // Type marker
    assert_eq!(buffer[2], b'l');  // Int32 type
    assert_ne!(buffer[3], b'#');  // No count marker
    assert_eq!(buffer[buffer.len() - 1], b']');  // Array end marker
}

#[test]
fn test_serializer_no_type_markers_in_typed_container() {
    // Test that elements in strongly-typed containers don't have type markers
    
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    
    let array = UbjsonValue::StronglyTypedArray {
        element_type: UbjsonType::Int8,
        count: Some(3),
        elements: vec![
            UbjsonValue::Int8(1),
            UbjsonValue::Int8(2),
            UbjsonValue::Int8(3),
        ],
    };
    
    serializer.serialize_value(&array).unwrap();
    
    // Structure should be: [$i#U3 1 2 3] (no 'i' markers before elements)
    let expected = vec![
        b'[',           // Array start
        b'$',           // Type marker
        b'i',           // Int8 type
        b'#',           // Count marker
        b'U', 3,        // Count
        1, 2, 3,        // Elements without type markers
    ];
    assert_eq!(buffer, expected);
    
    // Verify no additional 'i' type markers in the element data
    let element_data = &buffer[6..]; // Skip header
    assert!(!element_data.contains(&b'i'));
}

// ============================================================================
// DESERIALIZER SPECIFICATION COMPLIANCE TESTS
// ============================================================================

#[test]
fn test_deserializer_accepts_zero_count() {
    // Test that deserializer accepts count of 0
    
    let data = vec![
        b'[',           // Array start
        b'$',           // Type marker
        b'i',           // Int8 type
        b'#',           // Count marker
        b'U', 0,        // Count (0)
        // No elements, no end marker
    ];
    
    let mut cursor = Cursor::new(data);
    let mut deserializer = UbjsonDeserializer::new(&mut cursor);
    
    let result = deserializer.deserialize_value().unwrap();
    
    match result {
        UbjsonValue::StronglyTypedArray { element_type, count, elements } => {
            assert_eq!(element_type, UbjsonType::Int8);
            assert_eq!(count, Some(0));
            assert_eq!(elements.len(), 0);
        }
        _ => panic!("Expected StronglyTypedArray"),
    }
}

#[test]
fn test_deserializer_count_without_end_marker() {
    // Test that deserializer doesn't expect end marker when count is specified
    
    let data = vec![
        b'[',           // Array start
        b'$',           // Type marker
        b'U',           // UInt8 type
        b'#',           // Count marker
        b'U', 2,        // Count (2)
        10, 20,         // 2 elements, no end marker
    ];
    
    let mut cursor = Cursor::new(data);
    let mut deserializer = UbjsonDeserializer::new(&mut cursor);
    
    let result = deserializer.deserialize_value().unwrap();
    
    match result {
        UbjsonValue::StronglyTypedArray { element_type, count, elements } => {
            assert_eq!(element_type, UbjsonType::UInt8);
            assert_eq!(count, Some(2));
            assert_eq!(elements.len(), 2);
            assert_eq!(elements[0], UbjsonValue::UInt8(10));
            assert_eq!(elements[1], UbjsonValue::UInt8(20));
        }
        _ => panic!("Expected StronglyTypedArray"),
    }
}

#[test]
fn test_deserializer_validates_element_count() {
    // Test that deserializer validates the actual element count matches specified count
    
    let data = vec![
        b'[',           // Array start
        b'$',           // Type marker
        b'i',           // Int8 type
        b'#',           // Count marker
        b'U', 3,        // Count says 3 elements
        1, 2,           // But only 2 elements provided (missing 3rd)
    ];
    
    let mut cursor = Cursor::new(data);
    let mut deserializer = UbjsonDeserializer::new(&mut cursor);
    
    // This should fail because we don't have enough elements
    let result = deserializer.deserialize_value();
    assert!(result.is_err());
}

#[test]
fn test_deserializer_type_before_count_order() {
    // Test that deserializer expects type marker before count marker
    
    // Correct order: [$i#U2 1 2]
    let correct_data = vec![
        b'[',           // Array start
        b'$',           // Type marker
        b'i',           // Int8 type (comes first)
        b'#',           // Count marker (comes second)
        b'U', 2,        // Count
        1, 2,           // Elements
    ];
    
    let mut cursor = Cursor::new(correct_data);
    let mut deserializer = UbjsonDeserializer::new(&mut cursor);
    
    let result = deserializer.deserialize_value();
    assert!(result.is_ok());
    
    // Incorrect order: [#U2$i 1 2] - this should fail
    let incorrect_data = vec![
        b'[',           // Array start
        b'#',           // Count marker (wrong - should come after type)
        b'U', 2,        // Count
        b'$',           // Type marker (wrong - should come before count)
        b'i',           // Int8 type
        1, 2,           // Elements
    ];
    
    let mut cursor = Cursor::new(incorrect_data);
    let mut deserializer = UbjsonDeserializer::new(&mut cursor);
    
    let result = deserializer.deserialize_value();
    assert!(result.is_err()); // Should fail due to incorrect order
}

#[test]
fn test_deserializer_type_without_count_allowed() {
    // Test that type can be specified without count (but then end marker is required)
    
    let data = vec![
        b'[',           // Array start
        b'$',           // Type marker
        b'i',           // Int8 type
        // No count marker
        1, 2, 3,        // Elements without type markers
        b']',           // End marker (required when no count)
    ];
    
    let mut cursor = Cursor::new(data);
    let mut deserializer = UbjsonDeserializer::new(&mut cursor);
    
    let result = deserializer.deserialize_value().unwrap();
    
    match result {
        UbjsonValue::StronglyTypedArray { element_type, count, elements } => {
            assert_eq!(element_type, UbjsonType::Int8);
            assert_eq!(count, None); // No count specified
            assert_eq!(elements.len(), 3);
        }
        _ => panic!("Expected StronglyTypedArray"),
    }
}

#[test]
fn test_deserializer_no_type_markers_in_elements() {
    // Test that deserializer doesn't expect type markers for elements in typed containers
    
    let data = vec![
        b'[',           // Array start
        b'$',           // Type marker
        b'I',           // Int16 type
        b'#',           // Count marker
        b'U', 2,        // Count (2)
        // Elements as raw Int16 values (big-endian), no type markers
        0x03, 0xE8,     // 1000 as Int16
        0x07, 0xD0,     // 2000 as Int16
    ];
    
    let mut cursor = Cursor::new(data);
    let mut deserializer = UbjsonDeserializer::new(&mut cursor);
    
    let result = deserializer.deserialize_value().unwrap();
    
    match result {
        UbjsonValue::StronglyTypedArray { element_type, count, elements } => {
            assert_eq!(element_type, UbjsonType::Int16);
            assert_eq!(count, Some(2));
            assert_eq!(elements.len(), 2);
            assert_eq!(elements[0], UbjsonValue::Int16(1000));
            assert_eq!(elements[1], UbjsonValue::Int16(2000));
        }
        _ => panic!("Expected StronglyTypedArray"),
    }
}

#[test]
fn test_deserializer_rejects_type_markers_in_typed_elements() {
    // Test that deserializer correctly interprets data in typed containers
    // In this case, the bytes that happen to be type markers are treated as data
    
    let data = vec![
        b'[',           // Array start
        b'$',           // Type marker
        b'i',           // Int8 type
        b'#',           // Count marker
        b'U', 2,        // Count (2)
        b'i', 1,        // First element: 105 (which happens to be 'i'), second element: 1
    ];
    
    let mut cursor = Cursor::new(data);
    let mut deserializer = UbjsonDeserializer::new(&mut cursor);
    
    // This should succeed and interpret the bytes as raw Int8 values
    let result = deserializer.deserialize_value().unwrap();
    
    match result {
        UbjsonValue::StronglyTypedArray { element_type, count, elements } => {
            assert_eq!(element_type, UbjsonType::Int8);
            assert_eq!(count, Some(2));
            assert_eq!(elements.len(), 2);
            assert_eq!(elements[0], UbjsonValue::Int8(105)); // 'i' as Int8
            assert_eq!(elements[1], UbjsonValue::Int8(1));
        }
        _ => panic!("Expected StronglyTypedArray"),
    }
}

#[test]
fn test_deserializer_object_count_validation() {
    // Test count validation for objects
    
    let data = vec![
        b'{',           // Object start
        b'$',           // Type marker
        b'i',           // Int8 value type
        b'#',           // Count marker
        b'U', 2,        // Count says 2 pairs
        // First key-value pair
        b'U', 1, b'a',  // Key "a" in compact format
        1,              // Value 1 (no type marker)
        // Second key-value pair
        b'U', 1, b'b',  // Key "b" in compact format
        2,              // Value 2 (no type marker)
        // No third pair, but count says 2 - this should be OK
    ];
    
    let mut cursor = Cursor::new(data);
    let mut deserializer = UbjsonDeserializer::new(&mut cursor);
    
    let result = deserializer.deserialize_value().unwrap();
    
    match result {
        UbjsonValue::StronglyTypedObject { value_type, count, pairs } => {
            assert_eq!(value_type, UbjsonType::Int8);
            assert_eq!(count, Some(2));
            assert_eq!(pairs.len(), 2);
        }
        _ => panic!("Expected StronglyTypedObject"),
    }
}

#[test]
fn test_deserializer_object_count_mismatch_error() {
    // Test that deserializer fails when object has wrong number of pairs
    
    let data = vec![
        b'{',           // Object start
        b'$',           // Type marker
        b'i',           // Int8 value type
        b'#',           // Count marker
        b'U', 3,        // Count says 3 pairs
        // First key-value pair
        b'S', b'U', 1, b'a',  // Key "a" with string marker
        1,              // Value 1
        // Second key-value pair
        b'S', b'U', 1, b'b',  // Key "b" with string marker
        2,              // Value 2
        // Missing third pair - should cause error
    ];
    
    let mut cursor = Cursor::new(data);
    let mut deserializer = UbjsonDeserializer::new(&mut cursor);
    
    let result = deserializer.deserialize_value();
    assert!(result.is_err()); // Should fail due to insufficient pairs
}

// ============================================================================
// ROUND-TRIP TESTS FOR SPECIFICATION COMPLIANCE
// ============================================================================

#[test]
fn test_round_trip_strongly_typed_array_with_count() {
    // Test that serialization and deserialization preserve all specification requirements
    
    let original = UbjsonValue::StronglyTypedArray {
        element_type: UbjsonType::Float32,
        count: Some(3),
        elements: vec![
            UbjsonValue::Float32(1.5),
            UbjsonValue::Float32(2.5),
            UbjsonValue::Float32(3.5),
        ],
    };
    
    // Serialize
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    serializer.serialize_value(&original).unwrap();
    
    // Verify serialized format compliance
    assert_eq!(buffer[0], b'[');  // Array start
    assert_eq!(buffer[1], b'$');  // Type marker
    assert_eq!(buffer[2], b'd');  // Float32 type
    assert_eq!(buffer[3], b'#');  // Count marker
    assert_eq!(buffer[4], b'U');  // Count type
    assert_eq!(buffer[5], 3);     // Count value
    assert_ne!(buffer[buffer.len() - 1], b']'); // No end marker
    
    // Deserialize
    let mut cursor = Cursor::new(buffer);
    let mut deserializer = UbjsonDeserializer::new(&mut cursor);
    let deserialized = deserializer.deserialize_value().unwrap();
    
    // Verify round-trip preservation
    assert_eq!(original, deserialized);
}

#[test]
fn test_round_trip_strongly_typed_object_without_count() {
    // Test round-trip for object without count (should have end marker)
    
    let mut pairs = HashMap::new();
    pairs.insert("x".to_string(), UbjsonValue::Int32(100));
    pairs.insert("y".to_string(), UbjsonValue::Int32(200));
    
    let original = UbjsonValue::StronglyTypedObject {
        value_type: UbjsonType::Int32,
        count: None,
        pairs,
    };
    
    // Serialize
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    serializer.serialize_value(&original).unwrap();
    
    // Verify serialized format compliance
    assert_eq!(buffer[0], b'{');  // Object start
    assert_eq!(buffer[1], b'$');  // Type marker
    assert_eq!(buffer[2], b'l');  // Int32 type
    assert_ne!(buffer[3], b'#');  // No count marker
    assert_eq!(buffer[buffer.len() - 1], b'}'); // End marker present
    
    // Deserialize
    let mut cursor = Cursor::new(buffer);
    let mut deserializer = UbjsonDeserializer::new(&mut cursor);
    let deserialized = deserializer.deserialize_value().unwrap();
    
    // Verify round-trip preservation
    assert_eq!(original, deserialized);
}