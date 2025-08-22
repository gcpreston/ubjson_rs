use std::collections::HashMap;
use ubjson_rs::{UbjsonType, UbjsonValue};

#[test]
fn test_value_types() {
    assert_eq!(UbjsonValue::Null.get_type(), UbjsonType::Null);
    assert_eq!(UbjsonValue::Bool(true).get_type(), UbjsonType::True);
    assert_eq!(UbjsonValue::Bool(false).get_type(), UbjsonType::False);
    assert_eq!(UbjsonValue::Int32(42).get_type(), UbjsonType::Int32);
    assert_eq!(UbjsonValue::String("test".to_string()).get_type(), UbjsonType::String);
}

#[test]
fn test_type_checks() {
    let null_val = UbjsonValue::Null;
    let bool_val = UbjsonValue::Bool(true);
    let int_val = UbjsonValue::Int32(42);
    let float_val = UbjsonValue::Float64(3.14);
    let string_val = UbjsonValue::String("test".to_string());
    let array_val = UbjsonValue::Array(vec![]);

    assert!(null_val.is_null());
    assert!(bool_val.is_bool());
    assert!(int_val.is_number());
    assert!(int_val.is_integer());
    assert!(float_val.is_number());
    assert!(float_val.is_float());
    assert!(string_val.is_string());
    assert!(array_val.is_array());
}

#[test]
fn test_conversions() {
    assert_eq!(UbjsonValue::from(true), UbjsonValue::Bool(true));
    assert_eq!(UbjsonValue::from(42i32), UbjsonValue::Int32(42));
    assert_eq!(UbjsonValue::from(3.14f64), UbjsonValue::Float64(3.14));
    assert_eq!(UbjsonValue::from("test"), UbjsonValue::String("test".to_string()));
}

#[test]
fn test_container_length() {
    let empty_array = UbjsonValue::Array(vec![]);
    let array = UbjsonValue::Array(vec![UbjsonValue::Int32(1), UbjsonValue::Int32(2)]);
    let empty_object = UbjsonValue::Object(HashMap::new());
    
    assert_eq!(empty_array.len(), Some(0));
    assert_eq!(array.len(), Some(2));
    assert_eq!(empty_object.len(), Some(0));
    assert_eq!(UbjsonValue::Null.len(), None);
    
    assert!(empty_array.is_empty());
    assert!(!array.is_empty());
}