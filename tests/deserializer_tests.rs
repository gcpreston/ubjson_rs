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

    // Test object with non-string key
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
        data.push(b'S');
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
    
    // Key "users"
    data.push(b'S');
    data.push(b'U');
    data.push(5); // length
    data.extend_from_slice(b"users");
    
    // Value: array of user objects
    data.push(b'['); // Users array start
    
    // First user object
    data.push(b'{'); // User object start
    
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
    
    // Key "scores"
    data.push(b'S');
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
    
    // Key "name"
    data.push(b'S');
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"name");
    // Value "Jane"
    data.push(b'S');
    data.push(b'U');
    data.push(4); // length
    data.extend_from_slice(b"Jane");
    
    // Key "scores"
    data.push(b'S');
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