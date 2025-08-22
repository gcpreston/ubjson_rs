//! # UBJSON Library
//!
//! A Rust library for reading and writing Universal Binary JSON (UBJSON) format data.
//! UBJSON is a binary JSON format that provides efficient serialization and deserialization
//! of JSON-like data structures with type safety and container optimizations.
//!
//! ## Features
//!
//! - Full UBJSON specification support
//! - Container optimization for homogeneous collections
//! - Serde integration for automatic derive support
//! - Zero-copy deserialization where possible
//! - Comprehensive error handling and validation
//! - Performance optimizations for large datasets
//!
//! ## Example
//!
//! ```rust
//! use ubjson_rs::UbjsonValue;
//!
//! // Create UBJSON values
//! let null_value = UbjsonValue::Null;
//! let bool_value = UbjsonValue::Bool(true);
//! let int_value = UbjsonValue::Int32(42);
//! let string_value = UbjsonValue::String("Hello, UBJSON!".to_string());
//! 
//! // Check value types
//! assert!(null_value.is_null());
//! assert!(bool_value.is_bool());
//! assert!(int_value.is_number());
//! assert!(string_value.is_string());
//! ```

pub mod deserializer;
pub mod encoding;
pub mod error;
pub mod serializer;
pub mod types;
pub mod value;

// Re-export main types for convenience
pub use deserializer::UbjsonDeserializer;
pub use error::UbjsonError;
pub use serializer::UbjsonSerializer;
pub use types::UbjsonType;
pub use value::UbjsonValue;

// Placeholder for high-level API functions (will be implemented in later tasks)
pub fn to_vec<T>(_value: &T) -> Result<Vec<u8>, UbjsonError> {
    todo!("Will be implemented in task 12")
}

pub fn from_slice<T>(_slice: &[u8]) -> Result<T, UbjsonError> {
    todo!("Will be implemented in task 12")
}