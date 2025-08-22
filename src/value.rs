//! UBJSON value representation and manipulation.

use std::collections::HashMap;
use crate::types::UbjsonType;

/// Represents any UBJSON value including optimized containers.
#[derive(Debug, Clone, PartialEq)]
pub enum UbjsonValue {
    /// Null value
    Null,
    /// Boolean value
    Bool(bool),
    /// Signed 8-bit integer
    Int8(i8),
    /// Unsigned 8-bit integer
    UInt8(u8),
    /// Signed 16-bit integer
    Int16(i16),
    /// Signed 32-bit integer
    Int32(i32),
    /// Signed 64-bit integer
    Int64(i64),
    /// 32-bit floating point number
    Float32(f32),
    /// 64-bit floating point number
    Float64(f64),
    /// High-precision number as string
    HighPrecision(String),
    /// Single character
    Char(char),
    /// UTF-8 string
    String(String),
    /// Standard array with mixed types
    Array(Vec<UbjsonValue>),
    /// Standard object with mixed value types
    Object(HashMap<String, UbjsonValue>),
    /// Strongly-typed array optimization
    StronglyTypedArray {
        /// The type of all elements in the array
        element_type: UbjsonType,
        /// Optional count for optimization (None means uncounted)
        count: Option<usize>,
        /// The array elements (all must match element_type)
        elements: Vec<UbjsonValue>,
    },
    /// Strongly-typed object optimization
    StronglyTypedObject {
        /// The type of all values in the object
        value_type: UbjsonType,
        /// Optional count for optimization (None means uncounted)
        count: Option<usize>,
        /// The key-value pairs (all values must match value_type)
        pairs: HashMap<String, UbjsonValue>,
    },
}

impl UbjsonValue {
    /// Get the UBJSON type of this value.
    pub fn get_type(&self) -> UbjsonType {
        match self {
            UbjsonValue::Null => UbjsonType::Null,
            UbjsonValue::Bool(true) => UbjsonType::True,
            UbjsonValue::Bool(false) => UbjsonType::False,
            UbjsonValue::Int8(_) => UbjsonType::Int8,
            UbjsonValue::UInt8(_) => UbjsonType::UInt8,
            UbjsonValue::Int16(_) => UbjsonType::Int16,
            UbjsonValue::Int32(_) => UbjsonType::Int32,
            UbjsonValue::Int64(_) => UbjsonType::Int64,
            UbjsonValue::Float32(_) => UbjsonType::Float32,
            UbjsonValue::Float64(_) => UbjsonType::Float64,
            UbjsonValue::HighPrecision(_) => UbjsonType::HighPrecision,
            UbjsonValue::Char(_) => UbjsonType::Char,
            UbjsonValue::String(_) => UbjsonType::String,
            UbjsonValue::Array(_) => UbjsonType::ArrayStart,
            UbjsonValue::Object(_) => UbjsonType::ObjectStart,
            UbjsonValue::StronglyTypedArray { .. } => UbjsonType::ArrayStart,
            UbjsonValue::StronglyTypedObject { .. } => UbjsonType::ObjectStart,
        }
    }

    /// Check if this value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, UbjsonValue::Null)
    }

    /// Check if this value is a boolean.
    pub fn is_bool(&self) -> bool {
        matches!(self, UbjsonValue::Bool(_))
    }

    /// Check if this value is a number (integer, float, or high-precision).
    pub fn is_number(&self) -> bool {
        match self {
            UbjsonValue::Int8(_)
            | UbjsonValue::UInt8(_)
            | UbjsonValue::Int16(_)
            | UbjsonValue::Int32(_)
            | UbjsonValue::Int64(_)
            | UbjsonValue::Float32(_)
            | UbjsonValue::Float64(_)
            | UbjsonValue::HighPrecision(_) => true,
            _ => false,
        }
    }

    /// Check if this value is an integer.
    pub fn is_integer(&self) -> bool {
        match self {
            UbjsonValue::Int8(_)
            | UbjsonValue::UInt8(_)
            | UbjsonValue::Int16(_)
            | UbjsonValue::Int32(_)
            | UbjsonValue::Int64(_) => true,
            _ => false,
        }
    }

    /// Check if this value is a floating-point number.
    pub fn is_float(&self) -> bool {
        matches!(self, UbjsonValue::Float32(_) | UbjsonValue::Float64(_))
    }

    /// Check if this value is a string.
    pub fn is_string(&self) -> bool {
        matches!(self, UbjsonValue::String(_))
    }

    /// Check if this value is a character.
    pub fn is_char(&self) -> bool {
        matches!(self, UbjsonValue::Char(_))
    }

    /// Check if this value is an array (standard or strongly-typed).
    pub fn is_array(&self) -> bool {
        match self {
            UbjsonValue::Array(_) | UbjsonValue::StronglyTypedArray { .. } => true,
            _ => false,
        }
    }

    /// Check if this value is an object (standard or strongly-typed).
    pub fn is_object(&self) -> bool {
        match self {
            UbjsonValue::Object(_) | UbjsonValue::StronglyTypedObject { .. } => true,
            _ => false,
        }
    }

    /// Get the length of a container (array or object), or None for non-containers.
    pub fn len(&self) -> Option<usize> {
        match self {
            UbjsonValue::Array(arr) => Some(arr.len()),
            UbjsonValue::Object(obj) => Some(obj.len()),
            UbjsonValue::StronglyTypedArray { elements, .. } => Some(elements.len()),
            UbjsonValue::StronglyTypedObject { pairs, .. } => Some(pairs.len()),
            _ => None,
        }
    }

    /// Check if a container is empty, or false for non-containers.
    pub fn is_empty(&self) -> bool {
        self.len().map_or(false, |len| len == 0)
    }

    /// Convert a boolean to UbjsonValue.
    pub fn from_bool(value: bool) -> Self {
        UbjsonValue::Bool(value)
    }

    /// Convert a string to UbjsonValue.
    pub fn from_string<S: Into<String>>(value: S) -> Self {
        UbjsonValue::String(value.into())
    }

    /// Convert a character to UbjsonValue.
    pub fn from_char(value: char) -> Self {
        UbjsonValue::Char(value)
    }

    /// Create an empty array.
    pub fn empty_array() -> Self {
        UbjsonValue::Array(Vec::new())
    }

    /// Create an empty object.
    pub fn empty_object() -> Self {
        UbjsonValue::Object(HashMap::new())
    }

    /// Create a strongly-typed array with the given element type.
    pub fn strongly_typed_array(element_type: UbjsonType, elements: Vec<UbjsonValue>) -> Self {
        UbjsonValue::StronglyTypedArray {
            element_type,
            count: Some(elements.len()),
            elements,
        }
    }

    /// Create a strongly-typed object with the given value type.
    pub fn strongly_typed_object(
        value_type: UbjsonType,
        pairs: HashMap<String, UbjsonValue>,
    ) -> Self {
        UbjsonValue::StronglyTypedObject {
            value_type,
            count: Some(pairs.len()),
            pairs,
        }
    }
}

// Implement From traits for convenient conversion from Rust types
impl From<bool> for UbjsonValue {
    fn from(value: bool) -> Self {
        UbjsonValue::Bool(value)
    }
}

impl From<i8> for UbjsonValue {
    fn from(value: i8) -> Self {
        UbjsonValue::Int8(value)
    }
}

impl From<u8> for UbjsonValue {
    fn from(value: u8) -> Self {
        UbjsonValue::UInt8(value)
    }
}

impl From<i16> for UbjsonValue {
    fn from(value: i16) -> Self {
        UbjsonValue::Int16(value)
    }
}

impl From<i32> for UbjsonValue {
    fn from(value: i32) -> Self {
        UbjsonValue::Int32(value)
    }
}

impl From<i64> for UbjsonValue {
    fn from(value: i64) -> Self {
        UbjsonValue::Int64(value)
    }
}

impl From<f32> for UbjsonValue {
    fn from(value: f32) -> Self {
        UbjsonValue::Float32(value)
    }
}

impl From<f64> for UbjsonValue {
    fn from(value: f64) -> Self {
        UbjsonValue::Float64(value)
    }
}

impl From<char> for UbjsonValue {
    fn from(value: char) -> Self {
        UbjsonValue::Char(value)
    }
}

impl From<String> for UbjsonValue {
    fn from(value: String) -> Self {
        UbjsonValue::String(value)
    }
}

impl From<&str> for UbjsonValue {
    fn from(value: &str) -> Self {
        UbjsonValue::String(value.to_string())
    }
}

impl From<Vec<UbjsonValue>> for UbjsonValue {
    fn from(value: Vec<UbjsonValue>) -> Self {
        UbjsonValue::Array(value)
    }
}

impl From<HashMap<String, UbjsonValue>> for UbjsonValue {
    fn from(value: HashMap<String, UbjsonValue>) -> Self {
        UbjsonValue::Object(value)
    }
}

impl std::fmt::Display for UbjsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UbjsonValue::Null => write!(f, "null"),
            UbjsonValue::Bool(b) => write!(f, "{}", b),
            UbjsonValue::Int8(n) => write!(f, "{}", n),
            UbjsonValue::UInt8(n) => write!(f, "{}", n),
            UbjsonValue::Int16(n) => write!(f, "{}", n),
            UbjsonValue::Int32(n) => write!(f, "{}", n),
            UbjsonValue::Int64(n) => write!(f, "{}", n),
            UbjsonValue::Float32(n) => write!(f, "{}", n),
            UbjsonValue::Float64(n) => write!(f, "{}", n),
            UbjsonValue::HighPrecision(s) => write!(f, "{}", s),
            UbjsonValue::Char(c) => write!(f, "'{}'", c),
            UbjsonValue::String(s) => write!(f, "\"{}\"", s),
            UbjsonValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            UbjsonValue::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, value)?;
                }
                write!(f, "}}")
            }
            UbjsonValue::StronglyTypedArray {
                element_type,
                elements,
                ..
            } => {
                write!(f, "[{}; ", element_type)?;
                for (i, item) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            UbjsonValue::StronglyTypedObject {
                value_type, pairs, ..
            } => {
                write!(f, "{{{}; ", value_type)?;
                for (i, (key, value)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}