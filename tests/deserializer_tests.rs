use std::io::Cursor;
use ubjson_rs::{UbjsonDeserializer, UbjsonValue, UbjsonError};

#[test]
fn test_deserialize_all_primitive_types() {
    // Test null
    let data = vec![b'Z'];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(deserializer.deserialize_value().unwrap(), UbjsonValue::Null);

    // Test boolean true
    let data = vec![b'T'];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(deserializer.deserialize_value().unwrap(), UbjsonValue::Bool(true));

    // Test boolean false
    let data = vec![b'F'];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(deserializer.deserialize_value().unwrap(), UbjsonValue::Bool(false));

    // Test int8
    let data = vec![b'i', 42];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(deserializer.deserialize_value().unwrap(), UbjsonValue::Int8(42));

    // Test uint8
    let data = vec![b'U', 200];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(deserializer.deserialize_value().unwrap(), UbjsonValue::UInt8(200));
}

#[test]
fn test_deserialize_integers() {
    // Test int16
    let data = vec![b'I', 0x01, 0x00]; // 256 in big-endian
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(deserializer.deserialize_value().unwrap(), UbjsonValue::Int16(256));

    // Test int32
    let data = vec![b'l', 0x00, 0x01, 0x00, 0x00]; // 65536 in big-endian
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(deserializer.deserialize_value().unwrap(), UbjsonValue::Int32(65536));

    // Test int64
    let data = vec![b'L', 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]; // 4294967296 in big-endian
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(deserializer.deserialize_value().unwrap(), UbjsonValue::Int64(4294967296));
}

#[test]
fn test_deserialize_floats() {
    // Test float32 - value 1.5
    let data = vec![b'd', 0x3F, 0xC0, 0x00, 0x00];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    if let UbjsonValue::Float32(value) = deserializer.deserialize_value().unwrap() {
        assert!((value - 1.5).abs() < f32::EPSILON);
    } else {
        panic!("Expected Float32");
    }

    // Test float64 - value 2.5
    let data = vec![b'D', 0x40, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    if let UbjsonValue::Float64(value) = deserializer.deserialize_value().unwrap() {
        assert!((value - 2.5).abs() < f64::EPSILON);
    } else {
        panic!("Expected Float64");
    }
}

#[test]
fn test_deserialize_strings() {
    // Test simple ASCII string
    let data = vec![b'S', b'U', 5, b'H', b'e', b'l', b'l', b'o'];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(
        deserializer.deserialize_value().unwrap(),
        UbjsonValue::String("Hello".to_string())
    );

    // Test empty string
    let data = vec![b'S', b'U', 0];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(
        deserializer.deserialize_value().unwrap(),
        UbjsonValue::String("".to_string())
    );

    // Test Unicode string
    let unicode_str = "Hello 世界 🌍";
    let bytes = unicode_str.as_bytes();
    let mut data = vec![b'S', b'U', bytes.len() as u8];
    data.extend_from_slice(bytes);
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(
        deserializer.deserialize_value().unwrap(),
        UbjsonValue::String(unicode_str.to_string())
    );
}

#[test]
fn test_deserialize_chars() {
    // Test ASCII character
    let data = vec![b'C', b'A'];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(deserializer.deserialize_value().unwrap(), UbjsonValue::Char('A'));

    // Test Unicode character (Euro symbol)
    let data = vec![b'C', 0xE2, 0x82, 0xAC];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(deserializer.deserialize_value().unwrap(), UbjsonValue::Char('€'));

    // Test emoji character
    let data = vec![b'C', 0xF0, 0x9F, 0x8C, 0x8D];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(deserializer.deserialize_value().unwrap(), UbjsonValue::Char('🌍'));
}

#[test]
fn test_deserialize_high_precision() {
    // Test regular high-precision number
    let number_str = "123.456789012345678901234567890";
    let bytes = number_str.as_bytes();
    let mut data = vec![b'H', b'U', bytes.len() as u8];
    data.extend_from_slice(bytes);
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(
        deserializer.deserialize_value().unwrap(),
        UbjsonValue::HighPrecision(number_str.to_string())
    );

    // Test scientific notation
    let number_str = "1.23e-45";
    let bytes = number_str.as_bytes();
    let mut data = vec![b'H', b'U', bytes.len() as u8];
    data.extend_from_slice(bytes);
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(
        deserializer.deserialize_value().unwrap(),
        UbjsonValue::HighPrecision(number_str.to_string())
    );

    // Test negative number
    let number_str = "-999.999";
    let bytes = number_str.as_bytes();
    let mut data = vec![b'H', b'U', bytes.len() as u8];
    data.extend_from_slice(bytes);
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(
        deserializer.deserialize_value().unwrap(),
        UbjsonValue::HighPrecision(number_str.to_string())
    );
}

#[test]
fn test_deserialize_no_op_handling() {
    // Test single no-op followed by value
    let data = vec![b'N', b'T'];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(deserializer.deserialize_value().unwrap(), UbjsonValue::Bool(true));

    // Test multiple no-ops followed by value
    let data = vec![b'N', b'N', b'N', b'F'];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    assert_eq!(deserializer.deserialize_value().unwrap(), UbjsonValue::Bool(false));
}

#[test]
fn test_error_handling() {
    // Test invalid type marker
    let data = vec![0xFF];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UbjsonError::InvalidTypeMarker(0xFF)));

    // Test unexpected EOF
    let data = vec![b'i']; // Int8 marker but no data
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value();
    assert!(result.is_err());

    // Test invalid UTF-8 in string
    let data = vec![b'S', b'U', 2, 0xFF, 0xFE];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UbjsonError::InvalidUtf8(_)));
}

#[test]
fn test_high_precision_validation() {
    // Test empty high-precision number
    let data = vec![b'H', b'U', 0];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UbjsonError::InvalidHighPrecision(_)));

    // Test invalid high-precision number with letters
    let invalid_str = "not_a_number";
    let bytes = invalid_str.as_bytes();
    let mut data = vec![b'H', b'U', bytes.len() as u8];
    data.extend_from_slice(bytes);
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UbjsonError::InvalidHighPrecision(_)));

    // Test invalid high-precision number with multiple decimal points
    let invalid_str = "123.45.67";
    let bytes = invalid_str.as_bytes();
    let mut data = vec![b'H', b'U', bytes.len() as u8];
    data.extend_from_slice(bytes);
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UbjsonError::InvalidHighPrecision(_)));
}

#[test]
fn test_limits() {
    // Test depth limit
    let data = vec![b'Z'];
    let mut deserializer = UbjsonDeserializer::with_limits(Cursor::new(data), 0, 1000);
    let result = deserializer.deserialize_value();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UbjsonError::DepthLimitExceeded(0)));

    // Test that normal operation works with reasonable limits
    let data = vec![b'T'];
    let mut deserializer = UbjsonDeserializer::with_limits(Cursor::new(data), 10, 1000);
    assert_eq!(deserializer.deserialize_value().unwrap(), UbjsonValue::Bool(true));
}

#[test]
fn test_deserialize_containers() {
    // Test empty array
    let data = vec![b'[', b']'];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();
    assert_eq!(result, UbjsonValue::Array(vec![]));

    // Test empty object
    let data = vec![b'{', b'}'];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();
    assert_eq!(result, UbjsonValue::Object(std::collections::HashMap::new()));

    // Test simple array with mixed types
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
fn test_unexpected_container_end_markers() {
    // Test array end marker without start
    let data = vec![b']'];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UbjsonError::InvalidFormat(_)));

    // Test object end marker without start
    let data = vec![b'}'];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UbjsonError::InvalidFormat(_)));
}

#[test]
fn test_container_error_conditions() {
    // Test object with duplicate keys
    let mut data = vec![b'{']; // Object start
    
    // First "name" key
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"name");
    data.push(b'S');
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"John");
    
    // Second "name" key (duplicate)
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

    // Test object with invalid length marker for key
    let data = vec![
        b'{',           // Object start
        b'T',           // Invalid length marker (true boolean, not a valid length type)
        b'S', b'U', 5, b'v', b'a', b'l', b'u', b'e', // "value"
        b'}',           // Object end
    ];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UbjsonError::InvalidFormat(_)));
}

#[test]
fn test_container_depth_limits() {
    // Test array depth limit
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

    // Test object depth limit
    let mut data = vec![];
    let depth = 5;
    
    // Create nested objects: {"a": {"b": {"c": {"d": {"e": null}}}}}
    for i in 0..depth {
        data.push(b'{');
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
fn test_container_size_limits() {
    // Test array size limit
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

    // Test object size limit
    let mut data = vec![b'{']; // Object start
    
    let size_limit = 2;
    for i in 0..size_limit + 1 {
        // Key
        data.push(b'U');
        data.push(4); // length
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
    assert!(matches!(result.unwrap_err(), UbjsonError::SizeLimitExceeded(_)));
}

#[test]
fn test_complex_nested_containers() {
    // Test complex nested structure: {"users": [{"name": "John", "scores": [95, 87]}, {"name": "Jane", "scores": []}]}
    let mut data = vec![b'{']; // Root object start
    
    // Key "users" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(5); // length
    data.extend_from_slice(b"users");
    
    // Value: array of user objects
    data.push(b'['); // Users array start
    
    // First user object
    data.push(b'{'); // User object start
    
    // Key "name" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"name");
    // Value "John"
    data.push(b'S');
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"John");
    
    // Key "scores" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(6); // length
    data.extend_from_slice(b"scores");
    // Value: array [95, 87]
    data.push(b'['); // Scores array start
    data.push(b'i'); data.push(95);
    data.push(b'i'); data.push(87);
    data.push(b']'); // Scores array end
    
    data.push(b'}'); // User object end
    
    // Second user object
    data.push(b'{'); // User object start
    
    // Key "name" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"name");
    // Value "Jane"
    data.push(b'S');
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"Jane");
    
    // Key "scores" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(6); // length
    data.extend_from_slice(b"scores");
    // Value: empty array
    data.push(b'['); // Scores array start
    data.push(b']'); // Scores array end
    
    data.push(b'}'); // User object end
    
    data.push(b']'); // Users array end
    data.push(b'}'); // Root object end
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();
    
    // Verify the structure
    if let UbjsonValue::Object(root) = result {
        if let Some(UbjsonValue::Array(users)) = root.get("users") {
            assert_eq!(users.len(), 2);
            
            // Check first user
            if let UbjsonValue::Object(user1) = &users[0] {
                assert_eq!(user1.get("name"), Some(&UbjsonValue::String("John".to_string())));
                if let Some(UbjsonValue::Array(scores)) = user1.get("scores") {
                    assert_eq!(scores.len(), 2);
                    assert_eq!(scores[0], UbjsonValue::Int8(95));
                    assert_eq!(scores[1], UbjsonValue::Int8(87));
                } else {
                    panic!("Expected scores array for John");
                }
            } else {
                panic!("Expected first user to be an object");
            }
            
            // Check second user
            if let UbjsonValue::Object(user2) = &users[1] {
                assert_eq!(user2.get("name"), Some(&UbjsonValue::String("Jane".to_string())));
                if let Some(UbjsonValue::Array(scores)) = user2.get("scores") {
                    assert_eq!(scores.len(), 0);
                } else {
                    panic!("Expected scores array for Jane");
                }
            } else {
                panic!("Expected second user to be an object");
            }
        } else {
            panic!("Expected users array");
        }
    } else {
        panic!("Expected root object");
    }
}

#[test]
fn test_round_trip_with_serializer() {
    use ubjson_rs::UbjsonSerializer;
    use std::io::Cursor;

    // Test round-trip for various primitive types
    let test_values = vec![
        UbjsonValue::Null,
        UbjsonValue::Bool(true),
        UbjsonValue::Bool(false),
        UbjsonValue::Int8(-42),
        UbjsonValue::UInt8(200),
        UbjsonValue::Int16(-1000),
        UbjsonValue::Int32(123456),
        UbjsonValue::Int64(-9876543210),
        UbjsonValue::Float32(3.14159),
        UbjsonValue::Float64(2.718281828459045),
        UbjsonValue::Char('A'),
        UbjsonValue::Char('€'),
        UbjsonValue::String("Hello, World!".to_string()),
        UbjsonValue::String("Unicode: 世界 🌍".to_string()),
        UbjsonValue::HighPrecision("123.456789012345678901234567890".to_string()),
    ];

    for original_value in test_values {
        // Serialize
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        serializer.serialize_value(&original_value).unwrap();

        // Deserialize
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(buffer));
        let deserialized_value = deserializer.deserialize_value().unwrap();

        // Compare
        assert_eq!(original_value, deserialized_value);
    }

    // Test containers
    let mut container_values = vec![
        UbjsonValue::Array(vec![]),
        UbjsonValue::Array(vec![
            UbjsonValue::Int8(1),
            UbjsonValue::String("test".to_string()),
            UbjsonValue::Bool(true),
        ]),
        UbjsonValue::Object(std::collections::HashMap::new()),
    ];

    // Create a simple object
    let mut simple_obj = std::collections::HashMap::new();
    simple_obj.insert("key1".to_string(), UbjsonValue::Int8(42));
    simple_obj.insert("key2".to_string(), UbjsonValue::String("value".to_string()));
    container_values.push(UbjsonValue::Object(simple_obj));

    for original_value in container_values {
        // Serialize
        let mut buffer = Vec::new();
        let mut serializer = UbjsonSerializer::new(&mut buffer);
        serializer.serialize_value(&original_value).unwrap();

        // Deserialize
        let mut deserializer = UbjsonDeserializer::new(Cursor::new(buffer));
        let deserialized_value = deserializer.deserialize_value().unwrap();

        // Compare
        assert_eq!(original_value, deserialized_value);
    }
}

// ===== OBJECT COMPLEXITY TESTS =====

#[test]
fn test_deserialize_object_level_1_simple() {
    // Level 1: Simple object with one key-value pair
    // {"type": 3}
    let data = vec![
        b'{',           // Object start
        b'U', 4,        // Length: uint8(4) for "type" (no 'S' marker for keys per UBJSON spec)
        b't', b'y', b'p', b'e',  // Key: "type"
        b'U', 3,        // Value: uint8(3)
        b'}',           // Object end
    ];
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();

    let mut expected_map = std::collections::HashMap::new();
    expected_map.insert("type".to_string(), UbjsonValue::UInt8(3));
    let expected = UbjsonValue::Object(expected_map);
    assert_eq!(result, expected);
}

#[test]
fn test_deserialize_object_level_2_multiple_primitives() {
    // Level 2: Object with multiple primitive values of different types
    // {"id": 42, "name": "Alice", "active": true, "score": 95.5}
    let mut data = vec![b'{']; // Object start
    
    // Key "id" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(2); // length
    data.extend_from_slice(b"id");
    // Value 42
    data.push(b'i');
    data.push(42);
    
    // Key "name"
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"name");
    // Value "Alice"
    data.push(b'S');
    data.push(b'U');
    data.push(5); // length
    data.extend_from_slice(b"Alice");
    
    // Key "active"
    data.push(b'U');
    data.push(6); // length
    data.extend_from_slice(b"active");
    // Value true
    data.push(b'T');
    
    // Key "score"
    data.push(b'U');
    data.push(5); // length
    data.extend_from_slice(b"score");
    // Value 95.5 as float32
    data.push(b'd');
    data.extend_from_slice(&95.5f32.to_be_bytes());
    
    data.push(b'}'); // Object end
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();
    
    let mut expected_map = std::collections::HashMap::new();
    expected_map.insert("id".to_string(), UbjsonValue::Int8(42));
    expected_map.insert("name".to_string(), UbjsonValue::String("Alice".to_string()));
    expected_map.insert("active".to_string(), UbjsonValue::Bool(true));
    expected_map.insert("score".to_string(), UbjsonValue::Float32(95.5));
    let expected = UbjsonValue::Object(expected_map);
    assert_eq!(result, expected);
}

#[test]
fn test_deserialize_object_level_3_with_arrays() {
    // Level 3: Object containing arrays
    // {"tags": ["rust", "json"], "numbers": [1, 2, 3], "empty": []}
    let mut data = vec![b'{']; // Object start
    
    // Key "tags" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"tags");
    // Value: array ["rust", "json"]
    data.push(b'['); // Array start
    data.push(b'S'); data.push(b'U'); data.push(4); data.extend_from_slice(b"rust");
    data.push(b'S'); data.push(b'U'); data.push(4); data.extend_from_slice(b"json");
    data.push(b']'); // Array end
    
    // Key "numbers" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(7); // length
    data.extend_from_slice(b"numbers");
    // Value: array [1, 2, 3]
    data.push(b'['); // Array start
    data.push(b'i'); data.push(1);
    data.push(b'i'); data.push(2);
    data.push(b'i'); data.push(3);
    data.push(b']'); // Array end
    
    // Key "empty" (no 'S' marker for keys per UBJSON spec)
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
    expected_map.insert("tags".to_string(), UbjsonValue::Array(vec![
        UbjsonValue::String("rust".to_string()),
        UbjsonValue::String("json".to_string()),
    ]));
    expected_map.insert("numbers".to_string(), UbjsonValue::Array(vec![
        UbjsonValue::Int8(1),
        UbjsonValue::Int8(2),
        UbjsonValue::Int8(3),
    ]));
    expected_map.insert("empty".to_string(), UbjsonValue::Array(vec![]));
    let expected = UbjsonValue::Object(expected_map);
    assert_eq!(result, expected);
}

#[test]
fn test_deserialize_object_level_4_nested_objects() {
    // Level 4: Object containing nested objects
    // {"user": {"name": "Bob", "age": 30}, "config": {"debug": false}}
    let mut data = vec![b'{']; // Root object start
    
    // Key "user" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"user");
    // Value: nested object
    data.push(b'{'); // Nested object start
    
    // Key "name" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"name");
    // Value "Bob"
    data.push(b'S');
    data.push(b'U');
    data.push(3); // length
    data.extend_from_slice(b"Bob");
    
    // Key "age" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(3); // length
    data.extend_from_slice(b"age");
    // Value 30
    data.push(b'i');
    data.push(30);
    
    data.push(b'}'); // Nested object end
    
    // Key "config" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(6); // length
    data.extend_from_slice(b"config");
    // Value: nested object
    data.push(b'{'); // Nested object start
    
    // Key "debug" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(5); // length
    data.extend_from_slice(b"debug");
    // Value false
    data.push(b'F');
    
    data.push(b'}'); // Nested object end
    data.push(b'}'); // Root object end
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();
    
    let mut user_map = std::collections::HashMap::new();
    user_map.insert("name".to_string(), UbjsonValue::String("Bob".to_string()));
    user_map.insert("age".to_string(), UbjsonValue::Int8(30));
    
    let mut config_map = std::collections::HashMap::new();
    config_map.insert("debug".to_string(), UbjsonValue::Bool(false));
    
    let mut expected_map = std::collections::HashMap::new();
    expected_map.insert("user".to_string(), UbjsonValue::Object(user_map));
    expected_map.insert("config".to_string(), UbjsonValue::Object(config_map));
    let expected = UbjsonValue::Object(expected_map);
    assert_eq!(result, expected);
}

#[test]
fn test_deserialize_object_level_5_mixed_complex() {
    // Level 5: Complex object with mixed nested structures
    // {
    //   "metadata": {"version": "1.0", "author": "test"},
    //   "data": [{"id": 1, "values": [10, 20]}, {"id": 2, "values": []}],
    //   "settings": {"enabled": true, "threshold": 0.95}
    // }
    let mut data = vec![b'{']; // Root object start
    
    // Key "metadata" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(8); // length
    data.extend_from_slice(b"metadata");
    // Value: nested object
    data.push(b'{'); // Metadata object start
    
    // Key "version" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(7); // length
    data.extend_from_slice(b"version");
    // Value "1.0"
    data.push(b'S');
    data.push(b'U');
    data.push(3); // length
    data.extend_from_slice(b"1.0");
    
    // Key "author" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(6); // length
    data.extend_from_slice(b"author");
    // Value "test"
    data.push(b'S');
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"test");
    
    data.push(b'}'); // Metadata object end
    
    // Key "data" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"data");
    // Value: array of objects
    data.push(b'['); // Data array start
    
    // First data object
    data.push(b'{'); // Object start
    
    // Key "id" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(2); // length
    data.extend_from_slice(b"id");
    // Value 1
    data.push(b'i');
    data.push(1);
    
    // Key "values" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(6); // length
    data.extend_from_slice(b"values");
    // Value: array [10, 20]
    data.push(b'['); // Values array start
    data.push(b'i'); data.push(10);
    data.push(b'i'); data.push(20);
    data.push(b']'); // Values array end
    
    data.push(b'}'); // First data object end
    
    // Second data object
    data.push(b'{'); // Object start
    
    // Key "id" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(2); // length
    data.extend_from_slice(b"id");
    // Value 2
    data.push(b'i');
    data.push(2);
    
    // Key "values" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(6); // length
    data.extend_from_slice(b"values");
    // Value: empty array
    data.push(b'['); // Values array start
    data.push(b']'); // Values array end
    
    data.push(b'}'); // Second data object end
    data.push(b']'); // Data array end
    
    // Key "settings" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(8); // length
    data.extend_from_slice(b"settings");
    // Value: nested object
    data.push(b'{'); // Settings object start
    
    // Key "enabled" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(7); // length
    data.extend_from_slice(b"enabled");
    // Value true
    data.push(b'T');
    
    // Key "threshold" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(9); // length
    data.extend_from_slice(b"threshold");
    // Value 0.95 as float64
    data.push(b'D');
    data.extend_from_slice(&0.95f64.to_be_bytes());
    
    data.push(b'}'); // Settings object end
    data.push(b'}'); // Root object end
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();
    
    // Build expected structure
    let mut metadata_map = std::collections::HashMap::new();
    metadata_map.insert("version".to_string(), UbjsonValue::String("1.0".to_string()));
    metadata_map.insert("author".to_string(), UbjsonValue::String("test".to_string()));
    
    let mut data_obj1 = std::collections::HashMap::new();
    data_obj1.insert("id".to_string(), UbjsonValue::Int8(1));
    data_obj1.insert("values".to_string(), UbjsonValue::Array(vec![
        UbjsonValue::Int8(10),
        UbjsonValue::Int8(20),
    ]));
    
    let mut data_obj2 = std::collections::HashMap::new();
    data_obj2.insert("id".to_string(), UbjsonValue::Int8(2));
    data_obj2.insert("values".to_string(), UbjsonValue::Array(vec![]));
    
    let mut settings_map = std::collections::HashMap::new();
    settings_map.insert("enabled".to_string(), UbjsonValue::Bool(true));
    settings_map.insert("threshold".to_string(), UbjsonValue::Float64(0.95));
    
    let mut expected_map = std::collections::HashMap::new();
    expected_map.insert("metadata".to_string(), UbjsonValue::Object(metadata_map));
    expected_map.insert("data".to_string(), UbjsonValue::Array(vec![
        UbjsonValue::Object(data_obj1),
        UbjsonValue::Object(data_obj2),
    ]));
    expected_map.insert("settings".to_string(), UbjsonValue::Object(settings_map));
    let expected = UbjsonValue::Object(expected_map);
    assert_eq!(result, expected);
}

#[test]
fn test_deserialize_object_level_6_deeply_nested() {
    // Level 6: Deeply nested object structure (4 levels deep)
    // {"level1": {"level2": {"level3": {"level4": "deep_value"}}}}
    let mut data = vec![b'{']; // Root object start
    
    // Key "level1" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(6); // length
    data.extend_from_slice(b"level1");
    // Value: nested object
    data.push(b'{'); // Level 1 object start
    
    // Key "level2" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(6); // length
    data.extend_from_slice(b"level2");
    // Value: nested object
    data.push(b'{'); // Level 2 object start
    
    // Key "level3" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(6); // length
    data.extend_from_slice(b"level3");
    // Value: nested object
    data.push(b'{'); // Level 3 object start
    
    // Key "level4" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(6); // length
    data.extend_from_slice(b"level4");
    // Value "deep_value"
    data.push(b'S');
    data.push(b'U');
    data.push(10); // length
    data.extend_from_slice(b"deep_value");
    
    data.push(b'}'); // Level 3 object end
    data.push(b'}'); // Level 2 object end
    data.push(b'}'); // Level 1 object end
    data.push(b'}'); // Root object end
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();
    
    // Build expected nested structure
    let mut level4_map = std::collections::HashMap::new();
    level4_map.insert("level4".to_string(), UbjsonValue::String("deep_value".to_string()));
    
    let mut level3_map = std::collections::HashMap::new();
    level3_map.insert("level3".to_string(), UbjsonValue::Object(level4_map));
    
    let mut level2_map = std::collections::HashMap::new();
    level2_map.insert("level2".to_string(), UbjsonValue::Object(level3_map));
    
    let mut expected_map = std::collections::HashMap::new();
    expected_map.insert("level1".to_string(), UbjsonValue::Object(level2_map));
    let expected = UbjsonValue::Object(expected_map);
    assert_eq!(result, expected);
}

#[test]
fn test_deserialize_object_level_7_all_data_types() {
    // Level 7: Object containing all supported UBJSON data types
    // {
    //   "null_val": null,
    //   "bool_true": true,
    //   "bool_false": false,
    //   "int8_val": -42,
    //   "uint8_val": 200,
    //   "int16_val": -1000,
    //   "int32_val": 123456,
    //   "int64_val": -9876543210,
    //   "float32_val": 3.14159,
    //   "float64_val": 2.718281828459045,
    //   "char_val": 'A',
    //   "string_val": "Hello",
    //   "high_precision": "123.456789012345678901234567890"
    // }
    let mut data = vec![b'{']; // Object start
    
    // null_val: null (no 'S' marker for keys per UBJSON spec)
    data.push(b'U'); data.push(8); data.extend_from_slice(b"null_val");
    data.push(b'Z');
    
    // bool_true: true
    data.push(b'U'); data.push(9); data.extend_from_slice(b"bool_true");
    data.push(b'T');
    
    // bool_false: false
    data.push(b'U'); data.push(10); data.extend_from_slice(b"bool_false");
    data.push(b'F');
    
    // int8_val: -42
    data.push(b'U'); data.push(8); data.extend_from_slice(b"int8_val");
    data.push(b'i'); data.push((-42i8) as u8);
    
    // uint8_val: 200
    data.push(b'U'); data.push(9); data.extend_from_slice(b"uint8_val");
    data.push(b'U'); data.push(200);
    
    // int16_val: -1000
    data.push(b'U'); data.push(9); data.extend_from_slice(b"int16_val");
    data.push(b'I'); data.extend_from_slice(&(-1000i16).to_be_bytes());
    
    // int32_val: 123456
    data.push(b'U'); data.push(9); data.extend_from_slice(b"int32_val");
    data.push(b'l'); data.extend_from_slice(&123456i32.to_be_bytes());
    
    // int64_val: -9876543210
    data.push(b'U'); data.push(9); data.extend_from_slice(b"int64_val");
    data.push(b'L'); data.extend_from_slice(&(-9876543210i64).to_be_bytes());
    
    // float32_val: 3.14159
    data.push(b'U'); data.push(11); data.extend_from_slice(b"float32_val");
    data.push(b'd'); data.extend_from_slice(&3.14159f32.to_be_bytes());
    
    // float64_val: 2.718281828459045
    data.push(b'U'); data.push(11); data.extend_from_slice(b"float64_val");
    data.push(b'D'); data.extend_from_slice(&2.718281828459045f64.to_be_bytes());
    
    // char_val: 'A'
    data.push(b'U'); data.push(8); data.extend_from_slice(b"char_val");
    data.push(b'C'); data.push(b'A');
    
    // string_val: "Hello"
    data.push(b'U'); data.push(10); data.extend_from_slice(b"string_val");
    data.push(b'S'); data.push(b'U'); data.push(5); data.extend_from_slice(b"Hello");
    
    // high_precision: "123.456789012345678901234567890"
    let hp_str = "123.456789012345678901234567890";
    data.push(b'U'); data.push(14); data.extend_from_slice(b"high_precision");
    data.push(b'H'); data.push(b'U'); data.push(hp_str.len() as u8); data.extend_from_slice(hp_str.as_bytes());
    
    data.push(b'}'); // Object end
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();
    
    let mut expected_map = std::collections::HashMap::new();
    expected_map.insert("null_val".to_string(), UbjsonValue::Null);
    expected_map.insert("bool_true".to_string(), UbjsonValue::Bool(true));
    expected_map.insert("bool_false".to_string(), UbjsonValue::Bool(false));
    expected_map.insert("int8_val".to_string(), UbjsonValue::Int8(-42));
    expected_map.insert("uint8_val".to_string(), UbjsonValue::UInt8(200));
    expected_map.insert("int16_val".to_string(), UbjsonValue::Int16(-1000));
    expected_map.insert("int32_val".to_string(), UbjsonValue::Int32(123456));
    expected_map.insert("int64_val".to_string(), UbjsonValue::Int64(-9876543210));
    expected_map.insert("float32_val".to_string(), UbjsonValue::Float32(3.14159));
    expected_map.insert("float64_val".to_string(), UbjsonValue::Float64(2.718281828459045));
    expected_map.insert("char_val".to_string(), UbjsonValue::Char('A'));
    expected_map.insert("string_val".to_string(), UbjsonValue::String("Hello".to_string()));
    expected_map.insert("high_precision".to_string(), UbjsonValue::HighPrecision("123.456789012345678901234567890".to_string()));
    let expected = UbjsonValue::Object(expected_map);
    assert_eq!(result, expected);
}

#[test]
fn test_deserialize_object_level_8_large_object() {
    // Level 8: Large object with many keys (testing performance and memory usage)
    let mut data = vec![b'{']; // Object start
    let num_keys = 50;
    
    for i in 0..num_keys {
        // Key: "key_XX"
        let key = format!("key_{:02}", i);
        data.push(b'U');
        data.push(key.len() as u8);
        data.extend_from_slice(key.as_bytes());
        
        // Value: i as int8
        data.push(b'i');
        data.push(i as u8);
    }
    
    data.push(b'}'); // Object end
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();
    
    if let UbjsonValue::Object(obj) = result {
        assert_eq!(obj.len(), num_keys);
        
        // Verify a few specific keys
        assert_eq!(obj.get("key_00"), Some(&UbjsonValue::Int8(0)));
        assert_eq!(obj.get("key_25"), Some(&UbjsonValue::Int8(25)));
        assert_eq!(obj.get("key_49"), Some(&UbjsonValue::Int8(49)));
        
        // Verify all keys are present
        for i in 0..num_keys {
            let key = format!("key_{:02}", i);
            assert!(obj.contains_key(&key), "Missing key: {}", key);
            assert_eq!(obj.get(&key), Some(&UbjsonValue::Int8(i as i8)));
        }
    } else {
        panic!("Expected Object");
    }
}

#[test]
fn test_deserialize_object_with_unicode_keys() {
    // Test object with Unicode keys and values
    // {"名前": "田中", "年齢": 25, "🌟": "special"}
    let mut data = vec![b'{']; // Object start
    
    // Key "名前" (name in Japanese)
    let name_key = "名前";
    let name_key_bytes = name_key.as_bytes();
    data.push(b'U');
    data.push(name_key_bytes.len() as u8);
    data.extend_from_slice(name_key_bytes);
    // Value "田中"
    let name_value = "田中";
    let name_value_bytes = name_value.as_bytes();
    data.push(b'S');
    data.push(b'U');
    data.push(name_value_bytes.len() as u8);
    data.extend_from_slice(name_value_bytes);
    
    // Key "年齢" (age in Japanese)
    let age_key = "年齢";
    let age_key_bytes = age_key.as_bytes();
    data.push(b'U');
    data.push(age_key_bytes.len() as u8);
    data.extend_from_slice(age_key_bytes);
    // Value 25
    data.push(b'i');
    data.push(25);
    
    // Key "🌟" (star emoji)
    let star_key = "🌟";
    let star_key_bytes = star_key.as_bytes();
    data.push(b'U');
    data.push(star_key_bytes.len() as u8);
    data.extend_from_slice(star_key_bytes);
    // Value "special"
    data.push(b'S');
    data.push(b'U');
    data.push(7);
    data.extend_from_slice(b"special");
    
    data.push(b'}'); // Object end
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();
    
    let mut expected_map = std::collections::HashMap::new();
    expected_map.insert("名前".to_string(), UbjsonValue::String("田中".to_string()));
    expected_map.insert("年齢".to_string(), UbjsonValue::Int8(25));
    expected_map.insert("🌟".to_string(), UbjsonValue::String("special".to_string()));
    let expected = UbjsonValue::Object(expected_map);
    assert_eq!(result, expected);
}

#[test]
fn test_deserialize_object_with_binary_data_approaches() {
    use std::collections::HashMap;
    
    // Approach 1: Array of UInt8 values (most straightforward)
    // {"image_data": [255, 0, 171, 205], "format": "raw"}
    let mut data = vec![b'{']; // Object start
    
    // Key "image_data" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(10); // length
    data.extend_from_slice(b"image_data");
    
    // Value: array of bytes [255, 0, 171, 205]
    data.push(b'['); // Array start
    data.push(b'U'); data.push(255);  // UInt8(255)
    data.push(b'U'); data.push(0);    // UInt8(0)
    data.push(b'U'); data.push(171);  // UInt8(171)
    data.push(b'U'); data.push(205);  // UInt8(205)
    data.push(b']'); // Array end
    
    // Key "format" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(6); // length
    data.extend_from_slice(b"format");
    
    // Value "raw"
    data.push(b'S');
    data.push(b'U');
    data.push(3); // length
    data.extend_from_slice(b"raw");
    
    data.push(b'}'); // Object end
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();
    
    let mut expected_map = HashMap::new();
    expected_map.insert("image_data".to_string(), UbjsonValue::Array(vec![
        UbjsonValue::UInt8(255),
        UbjsonValue::UInt8(0),
        UbjsonValue::UInt8(171),
        UbjsonValue::UInt8(205),
    ]));
    expected_map.insert("format".to_string(), UbjsonValue::String("raw".to_string()));
    let expected = UbjsonValue::Object(expected_map);
    assert_eq!(result, expected);
}

#[test]
fn test_deserialize_object_with_base64_binary_data() {
    use std::collections::HashMap;
    
    // Approach 2: Base64 encoded binary data
    // {"data": "/wCrzQ==", "encoding": "base64", "size": 4}
    let mut data = vec![b'{']; // Object start
    
    // Key "data" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"data");
    
    // Value: Base64 string "/wCrzQ==" (represents [0xFF, 0x00, 0xAB, 0xCD])
    let base64_data = "/wCrzQ==";
    data.push(b'S');
    data.push(b'U');
    data.push(base64_data.len() as u8);
    data.extend_from_slice(base64_data.as_bytes());
    
    // Key "encoding" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(8); // length
    data.extend_from_slice(b"encoding");
    
    // Value "base64"
    data.push(b'S');
    data.push(b'U');
    data.push(6); // length
    data.extend_from_slice(b"base64");
    
    // Key "size" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"size");
    
    // Value 4 (original binary size)
    data.push(b'i');
    data.push(4);
    
    data.push(b'}'); // Object end
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();
    
    let mut expected_map = HashMap::new();
    expected_map.insert("data".to_string(), UbjsonValue::String("/wCrzQ==".to_string()));
    expected_map.insert("encoding".to_string(), UbjsonValue::String("base64".to_string()));
    expected_map.insert("size".to_string(), UbjsonValue::Int8(4));
    let expected = UbjsonValue::Object(expected_map);
    assert_eq!(result, expected);
}

#[test]
fn test_deserialize_object_with_hex_binary_data() {
    use std::collections::HashMap;
    
    // Approach 3: Hexadecimal string representation
    // {"checksum": "ff00abcd", "algorithm": "crc32"}
    let mut data = vec![b'{']; // Object start
    
    // Key "checksum" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(8); // length
    data.extend_from_slice(b"checksum");
    
    // Value: Hex string "ff00abcd"
    data.push(b'S');
    data.push(b'U');
    data.push(8); // length
    data.extend_from_slice(b"ff00abcd");
    
    // Key "algorithm" (no 'S' marker for keys per UBJSON spec)
    data.push(b'U');
    data.push(9); // length
    data.extend_from_slice(b"algorithm");
    
    // Value "crc32"
    data.push(b'S');
    data.push(b'U');
    data.push(5); // length
    data.extend_from_slice(b"crc32");
    
    data.push(b'}'); // Object end
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();
    
    let mut expected_map = HashMap::new();
    expected_map.insert("checksum".to_string(), UbjsonValue::String("ff00abcd".to_string()));
    expected_map.insert("algorithm".to_string(), UbjsonValue::String("crc32".to_string()));
    let expected = UbjsonValue::Object(expected_map);
    assert_eq!(result, expected);
}

#[test]
fn test_binary_data_real_world_example() {
    use std::collections::HashMap;
    
    // Real-world example: Image metadata with thumbnail data
    // {
    //   "filename": "photo.jpg",
    //   "width": 1920,
    //   "height": 1080,
    //   "thumbnail": [255, 216, 255, 224, 0, 16], // JPEG header bytes
    //   "thumbnail_format": "jpeg"
    // }
    let mut data = vec![b'{']; // Object start
    
    // filename (no 'S' marker for keys per UBJSON spec)
    data.push(b'U'); data.push(8); data.extend_from_slice(b"filename");
    data.push(b'S'); data.push(b'U'); data.push(9); data.extend_from_slice(b"photo.jpg");
    
    // width
    data.push(b'U'); data.push(5); data.extend_from_slice(b"width");
    data.push(b'I'); data.extend_from_slice(&1920i16.to_be_bytes());
    
    // height
    data.push(b'U'); data.push(6); data.extend_from_slice(b"height");
    data.push(b'I'); data.extend_from_slice(&1080i16.to_be_bytes());
    
    // thumbnail (JPEG header bytes: FF D8 FF E0 00 10)
    data.push(b'U'); data.push(9); data.extend_from_slice(b"thumbnail");
    data.push(b'['); // Array start
    data.push(b'U'); data.push(255); // 0xFF
    data.push(b'U'); data.push(216); // 0xD8
    data.push(b'U'); data.push(255); // 0xFF
    data.push(b'U'); data.push(224); // 0xE0
    data.push(b'U'); data.push(0);   // 0x00
    data.push(b'U'); data.push(16);  // 0x10
    data.push(b']'); // Array end
    
    // thumbnail_format
    data.push(b'U'); data.push(16); data.extend_from_slice(b"thumbnail_format");
    data.push(b'S'); data.push(b'U'); data.push(4); data.extend_from_slice(b"jpeg");
    
    data.push(b'}'); // Object end
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();
    
    let mut expected_map = HashMap::new();
    expected_map.insert("filename".to_string(), UbjsonValue::String("photo.jpg".to_string()));
    expected_map.insert("width".to_string(), UbjsonValue::Int16(1920));
    expected_map.insert("height".to_string(), UbjsonValue::Int16(1080));
    expected_map.insert("thumbnail".to_string(), UbjsonValue::Array(vec![
        UbjsonValue::UInt8(255), // 0xFF
        UbjsonValue::UInt8(216), // 0xD8
        UbjsonValue::UInt8(255), // 0xFF
        UbjsonValue::UInt8(224), // 0xE0
        UbjsonValue::UInt8(0),   // 0x00
        UbjsonValue::UInt8(16),  // 0x10
    ]));
    expected_map.insert("thumbnail_format".to_string(), UbjsonValue::String("jpeg".to_string()));
    let expected = UbjsonValue::Object(expected_map);
    assert_eq!(result, expected);
}