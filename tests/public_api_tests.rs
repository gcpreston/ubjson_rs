//! Tests for the high-level public API functions.

use ubjson_rs::{
    UbjsonValue, UbjsonError, SerializerBuilder, DeserializerBuilder,
    to_vec, to_writer, from_slice, from_reader,
    value_to_vec, value_to_writer, value_from_slice, value_from_reader,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::io::Cursor;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestStruct {
    name: String,
    age: u32,
    active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct NestedStruct {
    id: i32,
    data: TestStruct,
    tags: Vec<String>,
}

#[test]
fn test_to_vec_and_from_slice_primitive() {
    let value = 42i32;
    
    // Serialize to Vec<u8>
    let bytes = to_vec(&value).unwrap();
    assert!(!bytes.is_empty());
    
    // Deserialize back
    let deserialized: i32 = from_slice(&bytes).unwrap();
    assert_eq!(deserialized, value);
}

#[test]
fn test_to_vec_and_from_slice_string() {
    let value = "Hello, UBJSON!".to_string();
    
    // Serialize to Vec<u8>
    let bytes = to_vec(&value).unwrap();
    assert!(!bytes.is_empty());
    
    // Deserialize back
    let deserialized: String = from_slice(&bytes).unwrap();
    assert_eq!(deserialized, value);
}

#[test]
fn test_to_vec_and_from_slice_struct() {
    let value = TestStruct {
        name: "Alice".to_string(),
        age: 30,
        active: true,
    };
    
    // Serialize to Vec<u8>
    let bytes = to_vec(&value).unwrap();
    assert!(!bytes.is_empty());
    
    // Deserialize back
    let deserialized: TestStruct = from_slice(&bytes).unwrap();
    assert_eq!(deserialized, value);
}

#[test]
fn test_to_vec_and_from_slice_nested_struct() {
    let value = NestedStruct {
        id: 123,
        data: TestStruct {
            name: "Bob".to_string(),
            age: 25,
            active: false,
        },
        tags: vec!["rust".to_string(), "ubjson".to_string()],
    };
    
    // Serialize to Vec<u8>
    let bytes = to_vec(&value).unwrap();
    assert!(!bytes.is_empty());
    
    // Deserialize back
    let deserialized: NestedStruct = from_slice(&bytes).unwrap();
    assert_eq!(deserialized, value);
}

#[test]
fn test_to_vec_and_from_slice_vec() {
    let value = vec![1, 2, 3, 4, 5];
    
    // Serialize to Vec<u8>
    let bytes = to_vec(&value).unwrap();
    assert!(!bytes.is_empty());
    
    // Deserialize back
    let deserialized: Vec<i32> = from_slice(&bytes).unwrap();
    assert_eq!(deserialized, value);
}

#[test]
fn test_to_vec_and_from_slice_hashmap() {
    let mut value = HashMap::new();
    value.insert("key1".to_string(), 100);
    value.insert("key2".to_string(), 200);
    value.insert("key3".to_string(), 300);
    
    // Serialize to Vec<u8>
    let bytes = to_vec(&value).unwrap();
    assert!(!bytes.is_empty());
    
    // Deserialize back
    let deserialized: HashMap<String, i32> = from_slice(&bytes).unwrap();
    assert_eq!(deserialized, value);
}

#[test]
fn test_to_writer_and_from_reader() {
    let value = TestStruct {
        name: "Charlie".to_string(),
        age: 35,
        active: true,
    };
    
    // Serialize to writer
    let mut buffer = Vec::new();
    to_writer(&mut buffer, &value).unwrap();
    assert!(!buffer.is_empty());
    
    // Deserialize from reader
    let cursor = Cursor::new(buffer);
    let deserialized: TestStruct = from_reader(cursor).unwrap();
    assert_eq!(deserialized, value);
}

#[test]
fn test_value_to_vec_and_value_from_slice_primitives() {
    let test_cases = vec![
        UbjsonValue::Null,
        UbjsonValue::Bool(true),
        UbjsonValue::Bool(false),
        UbjsonValue::Int8(-42),
        UbjsonValue::UInt8(255),
        UbjsonValue::Int16(-1000),
        UbjsonValue::Int32(100000),
        UbjsonValue::Int64(-1000000000),
        UbjsonValue::Float32(3.14159),
        UbjsonValue::Float64(2.718281828459045),
        UbjsonValue::Char('π'),
        UbjsonValue::String("Hello, World!".to_string()),
        UbjsonValue::HighPrecision("3.141592653589793238462643383279502884197".to_string()),
    ];
    
    for value in test_cases {
        // Serialize to Vec<u8>
        let bytes = value_to_vec(&value).unwrap();
        assert!(!bytes.is_empty());
        
        // Deserialize back
        let deserialized = value_from_slice(&bytes).unwrap();
        assert_eq!(deserialized, value);
    }
}

#[test]
fn test_value_to_vec_and_value_from_slice_array() {
    let value = UbjsonValue::Array(vec![
        UbjsonValue::Int32(1),
        UbjsonValue::Int32(2),
        UbjsonValue::Int32(3),
        UbjsonValue::String("test".to_string()),
        UbjsonValue::Bool(true),
    ]);
    
    // Serialize to Vec<u8>
    let bytes = value_to_vec(&value).unwrap();
    assert!(!bytes.is_empty());
    
    // Deserialize back
    let deserialized = value_from_slice(&bytes).unwrap();
    assert_eq!(deserialized, value);
}

#[test]
fn test_value_to_vec_and_value_from_slice_object() {
    let mut map = HashMap::new();
    map.insert("null".to_string(), UbjsonValue::Null);
    map.insert("bool".to_string(), UbjsonValue::Bool(true));
    map.insert("number".to_string(), UbjsonValue::Int32(42));
    map.insert("string".to_string(), UbjsonValue::String("test".to_string()));
    
    let value = UbjsonValue::Object(map);
    
    // Serialize to Vec<u8>
    let bytes = value_to_vec(&value).unwrap();
    assert!(!bytes.is_empty());
    
    // Deserialize back
    let deserialized = value_from_slice(&bytes).unwrap();
    assert_eq!(deserialized, value);
}

#[test]
fn test_value_to_writer_and_value_from_reader() {
    let value = UbjsonValue::Array(vec![
        UbjsonValue::String("item1".to_string()),
        UbjsonValue::String("item2".to_string()),
        UbjsonValue::String("item3".to_string()),
    ]);
    
    // Serialize to writer
    let mut buffer = Vec::new();
    value_to_writer(&mut buffer, &value).unwrap();
    assert!(!buffer.is_empty());
    
    // Deserialize from reader
    let cursor = Cursor::new(buffer);
    let deserialized = value_from_reader(cursor).unwrap();
    assert_eq!(deserialized, value);
}

#[test]
fn test_serializer_builder_default() {
    let builder = SerializerBuilder::new();
    let value = vec![1, 2, 3, 4, 5];
    
    // Serialize using builder
    let bytes = builder.to_vec(&value).unwrap();
    assert!(!bytes.is_empty());
    
    // Deserialize back
    let deserialized: Vec<i32> = from_slice(&bytes).unwrap();
    assert_eq!(deserialized, value);
}

#[test]
fn test_serializer_builder_with_optimization() {
    let builder = SerializerBuilder::new()
        .with_container_optimization(true);
    
    let value = vec![1, 2, 3, 4, 5]; // Homogeneous array should be optimized
    
    // Serialize using builder with optimization
    let bytes_optimized = builder.clone().to_vec(&value).unwrap();
    
    // Serialize using builder without optimization
    let builder_no_opt = SerializerBuilder::new()
        .with_container_optimization(false);
    let bytes_standard = builder_no_opt.to_vec(&value).unwrap();
    
    // Both should deserialize to the same value
    let deserialized_opt: Vec<i32> = from_slice(&bytes_optimized).unwrap();
    let deserialized_std: Vec<i32> = from_slice(&bytes_standard).unwrap();
    
    assert_eq!(deserialized_opt, value);
    assert_eq!(deserialized_std, value);
    
    // Optimized version might be different size (could be smaller or larger depending on data)
    assert!(!bytes_optimized.is_empty());
    assert!(!bytes_standard.is_empty());
}

#[test]
fn test_serializer_builder_with_max_depth() {
    let builder = SerializerBuilder::new()
        .with_max_depth(2);
    
    // Create a deeply nested structure that should exceed the depth limit
    let deeply_nested = vec![vec![vec![1, 2, 3]]]; // 3 levels deep
    
    // This should fail due to depth limit
    let result = builder.to_vec(&deeply_nested);
    assert!(result.is_err());
    
    if let Err(UbjsonError::DepthLimitExceeded(depth)) = result {
        assert_eq!(depth, 2);
    } else {
        panic!("Expected DepthLimitExceeded error");
    }
}

#[test]
fn test_serializer_builder_value_methods() {
    let builder = SerializerBuilder::new()
        .with_container_optimization(true);
    
    let value = UbjsonValue::Array(vec![
        UbjsonValue::Int32(1),
        UbjsonValue::Int32(2),
        UbjsonValue::Int32(3),
    ]);
    
    // Test value_to_vec
    let bytes = builder.clone().value_to_vec(&value).unwrap();
    assert!(!bytes.is_empty());
    
    // Test value_to_writer
    let mut buffer = Vec::new();
    builder.value_to_writer(&mut buffer, &value).unwrap();
    assert_eq!(buffer, bytes);
    
    // Deserialize back
    let deserialized = value_from_slice(&bytes).unwrap();
    
    // When optimization is enabled, homogeneous arrays are serialized as strongly-typed arrays
    // So we need to check the logical equivalence rather than exact equality
    match (&deserialized, &value) {
        (UbjsonValue::StronglyTypedArray { elements, .. }, UbjsonValue::Array(original_elements)) => {
            assert_eq!(elements, original_elements);
        }
        _ => assert_eq!(deserialized, value),
    }
}

#[test]
fn test_deserializer_builder_default() {
    let value = TestStruct {
        name: "Dave".to_string(),
        age: 40,
        active: false,
    };
    
    // Serialize first
    let bytes = to_vec(&value).unwrap();
    
    // Deserialize using builder
    let builder = DeserializerBuilder::new();
    let deserialized: TestStruct = builder.from_slice(&bytes).unwrap();
    assert_eq!(deserialized, value);
}

#[test]
fn test_deserializer_builder_with_limits() {
    let builder = DeserializerBuilder::new()
        .with_max_depth(10)
        .with_max_size(1000);
    
    let value = vec![1, 2, 3, 4, 5];
    let bytes = to_vec(&value).unwrap();
    
    // Should work within limits
    let deserialized: Vec<i32> = builder.from_slice(&bytes).unwrap();
    assert_eq!(deserialized, value);
}

#[test]
fn test_deserializer_builder_size_limit_exceeded() {
    let builder = DeserializerBuilder::new()
        .with_max_size(2); // Very small limit
    
    let large_array = vec![1, 2, 3, 4, 5]; // 5 elements > limit of 2
    let bytes = to_vec(&large_array).unwrap();
    
    // Should fail due to size limit
    let result: Result<Vec<i32>, UbjsonError> = builder.from_slice(&bytes);
    assert!(result.is_err());
    
    if let Err(UbjsonError::SizeLimitExceeded(size)) = result {
        assert_eq!(size, 2);
    } else {
        panic!("Expected SizeLimitExceeded error, got: {:?}", result);
    }
}

#[test]
fn test_deserializer_builder_value_methods() {
    let value = UbjsonValue::Object({
        let mut map = HashMap::new();
        map.insert("test".to_string(), UbjsonValue::Int32(123));
        map
    });
    
    // Serialize first
    let bytes = value_to_vec(&value).unwrap();
    
    // Deserialize using builder
    let builder = DeserializerBuilder::new();
    
    // Test value_from_slice
    let deserialized = builder.clone().value_from_slice(&bytes).unwrap();
    assert_eq!(deserialized, value);
    
    // Test value_from_reader
    let cursor = Cursor::new(bytes);
    let deserialized = builder.value_from_reader(cursor).unwrap();
    assert_eq!(deserialized, value);
}

#[test]
fn test_round_trip_complex_data() {
    let mut complex_data = HashMap::new();
    complex_data.insert("users".to_string(), vec![
        TestStruct {
            name: "Alice".to_string(),
            age: 30,
            active: true,
        },
        TestStruct {
            name: "Bob".to_string(),
            age: 25,
            active: false,
        },
    ]);
    
    // Serialize
    let bytes = to_vec(&complex_data).unwrap();
    assert!(!bytes.is_empty());
    
    // Deserialize
    let deserialized: HashMap<String, Vec<TestStruct>> = from_slice(&bytes).unwrap();
    assert_eq!(deserialized, complex_data);
}

#[test]
fn test_empty_containers() {
    // Test empty Vec
    let empty_vec: Vec<i32> = vec![];
    let bytes = to_vec(&empty_vec).unwrap();
    let deserialized: Vec<i32> = from_slice(&bytes).unwrap();
    assert_eq!(deserialized, empty_vec);
    
    // Test empty HashMap
    let empty_map: HashMap<String, i32> = HashMap::new();
    let bytes = to_vec(&empty_map).unwrap();
    let deserialized: HashMap<String, i32> = from_slice(&bytes).unwrap();
    assert_eq!(deserialized, empty_map);
    
    // Test empty string
    let empty_string = String::new();
    let bytes = to_vec(&empty_string).unwrap();
    let deserialized: String = from_slice(&bytes).unwrap();
    assert_eq!(deserialized, empty_string);
}

#[test]
fn test_option_types() {
    // Test Some value
    let some_value: Option<i32> = Some(42);
    let bytes = to_vec(&some_value).unwrap();
    let deserialized: Option<i32> = from_slice(&bytes).unwrap();
    assert_eq!(deserialized, some_value);
    
    // Test None value
    let none_value: Option<i32> = None;
    let bytes = to_vec(&none_value).unwrap();
    let deserialized: Option<i32> = from_slice(&bytes).unwrap();
    assert_eq!(deserialized, none_value);
}

#[test]
fn test_error_handling_invalid_data() {
    // Test with invalid UBJSON data
    let invalid_data = vec![0xFF, 0xFF, 0xFF]; // Invalid type markers
    
    let result: Result<i32, UbjsonError> = from_slice(&invalid_data);
    assert!(result.is_err());
    
    let result = value_from_slice(&invalid_data);
    assert!(result.is_err());
}

#[test]
fn test_error_handling_truncated_data() {
    // Serialize a value first
    let value = "Hello, World!".to_string();
    let mut bytes = to_vec(&value).unwrap();
    
    // Truncate the data
    bytes.truncate(bytes.len() / 2);
    
    // Should fail to deserialize
    let result: Result<String, UbjsonError> = from_slice(&bytes);
    assert!(result.is_err());
}