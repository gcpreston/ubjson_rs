use std::collections::HashMap;
use std::io::Cursor;
use ubjson_rs::{UbjsonDeserializer, UbjsonSerializer, UbjsonValue};

#[test]
fn test_object_key_serialization_without_s_marker() {
    // Test that object keys are serialized without 'S' markers according to UBJSON spec
    let mut map = HashMap::new();
    map.insert("name".to_string(), UbjsonValue::String("John".to_string()));
    map.insert("age".to_string(), UbjsonValue::Int8(30));
    
    let object = UbjsonValue::Object(map);
    
    // Serialize the object
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    serializer.serialize_value(&object).unwrap();
    
    // Check that the first byte after object start is 'U' (length marker), not 'S' (string marker)
    let obj_start = buffer.iter().position(|&b| b == b'{').unwrap();
    assert_eq!(buffer[obj_start + 1], b'U', "Object keys should start with length marker 'U', not string marker 'S'");
    
    // Deserialize it back to verify round-trip works
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(&buffer));
    let result = deserializer.deserialize_value().unwrap();
    
    // Verify round-trip
    assert_eq!(result, object);
}

#[test]
fn test_object_key_deserialization_without_s_marker() {
    // Test that the deserializer can read object keys without 'S' markers
    let data = vec![
        b'{',           // Object start
        b'U', 4,        // Key "name" length (no 'S' marker)
        b'n', b'a', b'm', b'e',  // Key "name"
        b'S',           // Value type marker (string)
        b'U', 4,        // Value length
        b'J', b'o', b'h', b'n',  // Value "John"
        b'U', 3,        // Key "age" length (no 'S' marker)
        b'a', b'g', b'e',        // Key "age"
        b'i', 30,       // Value int8(30)
        b'}',           // Object end
    ];
    
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(data));
    let result = deserializer.deserialize_value().unwrap();
    
    let mut expected_map = HashMap::new();
    expected_map.insert("name".to_string(), UbjsonValue::String("John".to_string()));
    expected_map.insert("age".to_string(), UbjsonValue::Int8(30));
    let expected = UbjsonValue::Object(expected_map);
    
    assert_eq!(result, expected);
}

#[test]
fn test_nested_object_key_format() {
    // Test nested objects to ensure all levels use correct key format
    let mut inner_map = HashMap::new();
    inner_map.insert("id".to_string(), UbjsonValue::Int8(1));
    
    let mut outer_map = HashMap::new();
    outer_map.insert("user".to_string(), UbjsonValue::Object(inner_map));
    
    let object = UbjsonValue::Object(outer_map);
    
    // Serialize
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    serializer.serialize_value(&object).unwrap();
    
    // Deserialize and verify round-trip
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(&buffer));
    let result = deserializer.deserialize_value().unwrap();
    
    assert_eq!(result, object);
}

#[test]
fn test_empty_object_key_format() {
    // Test empty object
    let object = UbjsonValue::Object(HashMap::new());
    
    // Serialize
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    serializer.serialize_value(&object).unwrap();
    
    // Should be just { }
    assert_eq!(buffer, vec![b'{', b'}']);
    
    // Deserialize and verify round-trip
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(&buffer));
    let result = deserializer.deserialize_value().unwrap();
    
    assert_eq!(result, object);
}

#[test]
fn test_object_with_various_value_types() {
    // Test object with different value types to ensure only keys omit 'S' markers
    let mut map = HashMap::new();
    map.insert("null".to_string(), UbjsonValue::Null);
    map.insert("bool".to_string(), UbjsonValue::Bool(true));
    map.insert("int".to_string(), UbjsonValue::Int8(42));
    map.insert("string".to_string(), UbjsonValue::String("test".to_string()));
    
    let object = UbjsonValue::Object(map);
    
    // Serialize
    let mut buffer = Vec::new();
    let mut serializer = UbjsonSerializer::new(&mut buffer);
    serializer.serialize_value(&object).unwrap();
    
    // Deserialize and verify round-trip
    let mut deserializer = UbjsonDeserializer::new(Cursor::new(&buffer));
    let result = deserializer.deserialize_value().unwrap();
    
    assert_eq!(result, object);
}