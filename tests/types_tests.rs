use ubjson_rs::UbjsonType;

#[test]
fn test_type_conversion() {
    assert_eq!(UbjsonType::from_byte(b'Z').unwrap(), UbjsonType::Null);
    assert_eq!(UbjsonType::from_byte(b'T').unwrap(), UbjsonType::True);
    assert_eq!(UbjsonType::from_byte(b'F').unwrap(), UbjsonType::False);
    assert_eq!(UbjsonType::from_byte(b'i').unwrap(), UbjsonType::Int8);
    assert_eq!(UbjsonType::from_byte(b'S').unwrap(), UbjsonType::String);
    assert_eq!(UbjsonType::from_byte(b'[').unwrap(), UbjsonType::ArrayStart);
    assert_eq!(UbjsonType::from_byte(b'{').unwrap(), UbjsonType::ObjectStart);
    
    assert!(UbjsonType::from_byte(b'X').is_err());
}

#[test]
fn test_to_byte() {
    assert_eq!(UbjsonType::Null.to_byte(), b'Z');
    assert_eq!(UbjsonType::True.to_byte(), b'T');
    assert_eq!(UbjsonType::String.to_byte(), b'S');
    assert_eq!(UbjsonType::ArrayStart.to_byte(), b'[');
}

#[test]
fn test_type_classification() {
    assert!(UbjsonType::Null.is_primitive());
    assert!(UbjsonType::String.is_primitive());
    assert!(!UbjsonType::ArrayStart.is_primitive());
    
    assert!(UbjsonType::ArrayStart.is_container_start());
    assert!(UbjsonType::ObjectStart.is_container_start());
    assert!(!UbjsonType::Null.is_container_start());
    
    assert!(UbjsonType::Int32.is_numeric());
    assert!(UbjsonType::Float64.is_numeric());
    assert!(!UbjsonType::String.is_numeric());
    
    assert!(UbjsonType::Int32.is_integer());
    assert!(!UbjsonType::Float32.is_integer());
    
    assert!(UbjsonType::Float32.is_float());
    assert!(!UbjsonType::Int32.is_float());
}