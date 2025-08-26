//! Tests for serde integration with UBJSON serialization and deserialization.

#[cfg(feature = "serde")]
mod serde_tests {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use ubjson_rs::{UbjsonSerializer, UbjsonDeserializer, UbjsonError};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Person {
        name: String,
        age: u32,
        active: bool,
        height: f64,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Company {
        name: String,
        employees: Vec<Person>,
        founded: i32,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    enum Status {
        Active,
        Inactive,
        Pending(String),
        Complex { code: i32, message: String },
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        status: Status,
        tags: Vec<String>,
        metadata: HashMap<String, i32>,
    }

    #[test]
    fn test_serialize_deserialize_primitives() {
        // Test boolean
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        true.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: bool = bool::deserialize(deserializer).unwrap();
        assert_eq!(result, true);

        // Test integer
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        42i32.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: i32 = i32::deserialize(deserializer).unwrap();
        assert_eq!(result, 42);

        // Test float
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        3.14f64.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: f64 = f64::deserialize(deserializer).unwrap();
        assert!((result - 3.14).abs() < 0.0001);

        // Test string
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        "hello".serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: String = String::deserialize(deserializer).unwrap();
        assert_eq!(result, "hello");

        // Test character
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        'A'.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: char = char::deserialize(deserializer).unwrap();
        assert_eq!(result, 'A');
    }

    #[test]
    fn test_serialize_deserialize_option() {
        // Test Some
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        Some(42i32).serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: Option<i32> = Option::deserialize(deserializer).unwrap();
        assert_eq!(result, Some(42));

        // Test None
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        None::<i32>.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: Option<i32> = Option::deserialize(deserializer).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_serialize_deserialize_vec() {
        let original = vec![1, 2, 3, 4, 5];
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: Vec<i32> = Vec::deserialize(deserializer).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_serialize_deserialize_hashmap() {
        let mut original = HashMap::new();
        original.insert("one".to_string(), 1);
        original.insert("two".to_string(), 2);
        original.insert("three".to_string(), 3);
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: HashMap<String, i32> = HashMap::deserialize(deserializer).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_serialize_deserialize_struct() {
        let original = Person {
            name: "John Doe".to_string(),
            age: 30,
            active: true,
            height: 5.9,
        };
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: Person = Person::deserialize(deserializer).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_serialize_deserialize_struct_optional_keys() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Event {
            name: String,
            location: Option<String>
        }

        let original_with_location = Event {
            name: "Birthday party".to_string(),
            location: Some("My house".to_string())
        };

        let original_without_location = Event {
            name: "Going away party".to_string(),
            location: None
        };

        let mut buffer_with_location = Vec::new();
        let serializer_with_location = UbjsonSerializer::new(&mut buffer_with_location);
        original_with_location.serialize(serializer_with_location).unwrap();
        
        let mut buffer_without_location = Vec::new();
        let serializer_without_location = UbjsonSerializer::new(&mut buffer_without_location);
        original_without_location.serialize(serializer_without_location).unwrap();

        let deserializer_with_location = UbjsonDeserializer::new(buffer_with_location.as_slice());
        let result_with_location: Event = Event::deserialize(deserializer_with_location).unwrap();
        assert_eq!(result_with_location, original_with_location);

        let deserializer_without_location = UbjsonDeserializer::new(buffer_without_location.as_slice());
        let result_without_location: Event = Event::deserialize(deserializer_without_location).unwrap();
        assert_eq!(result_without_location, original_without_location);
    }

    #[test]
    fn test_serialize_deserialize_nested_struct() {
        let original = Company {
            name: "Acme Corp".to_string(),
            employees: vec![
                Person {
                    name: "Alice".to_string(),
                    age: 25,
                    active: true,
                    height: 5.6,
                },
                Person {
                    name: "Bob".to_string(),
                    age: 35,
                    active: false,
                    height: 6.0,
                },
            ],
            founded: 1990,
        };
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: Company = Company::deserialize(deserializer).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_serialize_deserialize_enum_unit_variant() {
        let original = Status::Active;
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: Status = Status::deserialize(deserializer).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_serialize_deserialize_enum_newtype_variant() {
        let original = Status::Pending("waiting for approval".to_string());
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: Status = Status::deserialize(deserializer).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_serialize_deserialize_enum_struct_variant() {
        let original = Status::Complex {
            code: 404,
            message: "Not Found".to_string(),
        };
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: Status = Status::deserialize(deserializer).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_serialize_deserialize_complex_struct() {
        // First test just the enum by itself
        let original_enum = Status::Complex {
            code: 200,
            message: "OK".to_string(),
        };
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original_enum.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result_enum: Status = Status::deserialize(deserializer).unwrap();
        assert_eq!(result_enum, original_enum);
        
        // Now test the full struct
        let mut metadata = HashMap::new();
        metadata.insert("priority".to_string(), 1);
        metadata.insert("version".to_string(), 2);
        
        let original = TestStruct {
            status: Status::Complex {
                code: 200,
                message: "OK".to_string(),
            },
            tags: vec!["important".to_string(), "urgent".to_string()],
            metadata,
        };
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: TestStruct = TestStruct::deserialize(deserializer).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_serialize_deserialize_tuple() {
        let original = (42, "hello".to_string(), true, 3.14);
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: (i32, String, bool, f64) = <(i32, String, bool, f64)>::deserialize(deserializer).unwrap();
        assert_eq!(result.0, original.0);
        assert_eq!(result.1, original.1);
        assert_eq!(result.2, original.2);
        assert!((result.3 - original.3).abs() < 0.0001);
    }

    #[test]
    fn test_serialize_deserialize_bytes() {
        let original = vec![0u8, 1, 2, 3, 255];
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original.as_slice().serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: Vec<u8> = Vec::deserialize(deserializer).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_serialize_deserialize_unit() {
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        ().serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: () = <()>::deserialize(deserializer).unwrap();
        assert_eq!(result, ());
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct UnitStruct;

    #[test]
    fn test_serialize_deserialize_unit_struct() {
        let original = UnitStruct;
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: UnitStruct = UnitStruct::deserialize(deserializer).unwrap();
        assert_eq!(result, original);
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct NewtypeStruct(i32);

    #[test]
    fn test_serialize_deserialize_newtype_struct() {
        let original = NewtypeStruct(42);
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: NewtypeStruct = NewtypeStruct::deserialize(deserializer).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_serialize_deserialize_large_numbers() {
        // Test u64 that fits in i64
        let original_u64 = 9223372036854775807u64; // i64::MAX as u64
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original_u64.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: u64 = u64::deserialize(deserializer).unwrap();
        assert_eq!(result, original_u64);

        // Test u64 that doesn't fit in i64 (should use high-precision)
        let original_large = u64::MAX;
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original_large.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: u64 = u64::deserialize(deserializer).unwrap();
        assert_eq!(result, original_large);
    }

    #[test]
    fn test_serialize_deserialize_unicode() {
        let original = "Hello, 世界! 🌍".to_string();
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: String = String::deserialize(deserializer).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_serialize_deserialize_empty_collections() {
        // Empty Vec
        let original_vec: Vec<i32> = vec![];
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original_vec.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: Vec<i32> = Vec::deserialize(deserializer).unwrap();
        assert_eq!(result, original_vec);

        // Empty HashMap
        let original_map: HashMap<String, i32> = HashMap::new();
        
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        original_map.serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: HashMap<String, i32> = HashMap::deserialize(deserializer).unwrap();
        assert_eq!(result, original_map);
    }

    #[test]
    fn test_round_trip_consistency() {
        // Test that multiple round trips produce the same result
        let original = Company {
            name: "Test Corp".to_string(),
            employees: vec![
                Person {
                    name: "Employee 1".to_string(),
                    age: 25,
                    active: true,
                    height: 5.8,
                },
            ],
            founded: 2000,
        };

        let mut buffer1 = Vec::new();
        let serializer1 = UbjsonSerializer::new(&mut buffer1);
        original.serialize(serializer1).unwrap();
        
        let deserializer1 = UbjsonDeserializer::new(buffer1.as_slice());
        let intermediate: Company = Company::deserialize(deserializer1).unwrap();
        
        let mut buffer2 = Vec::new();
        let serializer2 = UbjsonSerializer::new(&mut buffer2);
        intermediate.serialize(serializer2).unwrap();
        
        let deserializer2 = UbjsonDeserializer::new(buffer2.as_slice());
        let final_result: Company = Company::deserialize(deserializer2).unwrap();
        
        assert_eq!(original, final_result);
        // Note: Binary representation may differ due to HashMap field ordering, but the data should be identical
    }

    #[test]
    fn test_error_handling() {
        // Test deserializing invalid data
        let invalid_data = vec![0xFF]; // Invalid type marker
        let deserializer = UbjsonDeserializer::new(invalid_data.as_slice());
        let result: Result<i32, UbjsonError> = i32::deserialize(deserializer);
        assert!(result.is_err());

        // Test deserializing wrong type
        let mut buffer = Vec::new();
        let serializer = UbjsonSerializer::new(&mut buffer);
        "not a number".serialize(serializer).unwrap();
        
        let deserializer = UbjsonDeserializer::new(buffer.as_slice());
        let result: Result<i32, UbjsonError> = i32::deserialize(deserializer);
        assert!(result.is_err());
    }
}