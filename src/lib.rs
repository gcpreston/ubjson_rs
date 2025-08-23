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
//! ## Quick Start
//!
//! ### Serializing and Deserializing with Serde
//!
//! ```rust
//! use serde::{Serialize, Deserialize};
//! use ubjson_rs::{to_vec, from_slice};
//!
//! #[derive(Serialize, Deserialize, PartialEq, Debug)]
//! struct Person {
//!     name: String,
//!     age: u32,
//! }
//!
//! let person = Person {
//!     name: "Alice".to_string(),
//!     age: 30,
//! };
//!
//! // Serialize to UBJSON
//! let bytes = to_vec(&person).unwrap();
//!
//! // Deserialize from UBJSON
//! let deserialized: Person = from_slice(&bytes).unwrap();
//! assert_eq!(person, deserialized);
//! ```
//!
//! ### Working with UbjsonValue
//!
//! ```rust
//! use ubjson_rs::{UbjsonValue, value_to_vec, value_from_slice};
//!
//! // Create UBJSON values
//! let value = UbjsonValue::Array(vec![
//!     UbjsonValue::String("Hello".to_string()),
//!     UbjsonValue::Int32(42),
//!     UbjsonValue::Bool(true),
//! ]);
//!
//! // Serialize to bytes
//! let bytes = value_to_vec(&value).unwrap();
//!
//! // Deserialize back
//! let deserialized = value_from_slice(&bytes).unwrap();
//! assert_eq!(value, deserialized);
//! ```
//!
//! ### Using Builder Pattern for Configuration
//!
//! ```rust
//! use ubjson_rs::{SerializerBuilder, DeserializerBuilder};
//!
//! let data = vec![1, 2, 3, 4, 5];
//!
//! // Serialize with container optimization
//! let bytes = SerializerBuilder::new()
//!     .with_container_optimization(true)
//!     .with_max_depth(100)
//!     .to_vec(&data)
//!     .unwrap();
//!
//! // Deserialize with custom limits
//! let deserialized: Vec<i32> = DeserializerBuilder::new()
//!     .with_max_size(10000)
//!     .with_max_depth(100)
//!     .from_slice(&bytes)
//!     .unwrap();
//!
//! assert_eq!(data, deserialized);
//! ```

pub mod deserializer;
pub mod encoding;
pub mod error;
pub mod serializer;
#[cfg(feature = "serde")]
pub mod serde_impl;
pub mod types;
pub mod value;

// Re-export main types for convenience
pub use deserializer::UbjsonDeserializer;
pub use error::{UbjsonError, Result};
pub use serializer::UbjsonSerializer;
pub use types::UbjsonType;
pub use value::UbjsonValue;

// High-level convenience functions for serde integration
#[cfg(feature = "serde")]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: serde::Serialize,
{
    let mut buffer = Vec::new();
    to_writer(&mut buffer, value)?;
    Ok(buffer)
}

#[cfg(feature = "serde")]
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: std::io::Write,
    T: serde::Serialize,
{
    let serializer = UbjsonSerializer::new(writer);
    value.serialize(serializer)
}

#[cfg(feature = "serde")]
pub fn from_slice<'a, T>(slice: &'a [u8]) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    from_reader(slice)
}

#[cfg(feature = "serde")]
pub fn from_reader<R, T>(reader: R) -> Result<T>
where
    R: std::io::Read,
    T: serde::de::DeserializeOwned,
{
    let deserializer = UbjsonDeserializer::new(reader);
    T::deserialize(deserializer)
}

// Value-based serialization functions for UbjsonValue
pub fn value_to_vec(value: &UbjsonValue) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    value_to_writer(&mut buffer, value)?;
    Ok(buffer)
}

pub fn value_to_writer<W>(writer: W, value: &UbjsonValue) -> Result<()>
where
    W: std::io::Write,
{
    let mut serializer = UbjsonSerializer::new(writer);
    serializer.serialize_value(value)
}

pub fn value_from_slice(slice: &[u8]) -> Result<UbjsonValue> {
    value_from_reader(slice)
}

pub fn value_from_reader<R>(reader: R) -> Result<UbjsonValue>
where
    R: std::io::Read,
{
    let mut deserializer = UbjsonDeserializer::new(reader);
    deserializer.deserialize_value()
}

/// Builder for configuring UBJSON serialization options.
#[derive(Debug, Clone)]
pub struct SerializerBuilder {
    optimize_containers: bool,
    max_depth: usize,
}

impl Default for SerializerBuilder {
    fn default() -> Self {
        Self {
            optimize_containers: false,
            max_depth: UbjsonSerializer::<std::io::Sink>::DEFAULT_MAX_DEPTH,
        }
    }
}

impl SerializerBuilder {
    /// Create a new serializer builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable container optimization.
    /// 
    /// When enabled, homogeneous arrays and objects will be serialized using
    /// UBJSON's strongly-typed container format for better efficiency.
    pub fn with_container_optimization(mut self, optimize: bool) -> Self {
        self.optimize_containers = optimize;
        self
    }

    /// Set the maximum nesting depth to prevent stack overflow.
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Build a serializer with the configured options for the given writer.
    pub fn build<W: std::io::Write>(self, writer: W) -> UbjsonSerializer<W> {
        UbjsonSerializer::with_settings(writer, self.optimize_containers, self.max_depth)
    }

    /// Serialize a value to a Vec<u8> using the configured options.
    #[cfg(feature = "serde")]
    pub fn to_vec<T>(self, value: &T) -> Result<Vec<u8>>
    where
        T: serde::Serialize,
    {
        let mut buffer = Vec::new();
        self.to_writer(&mut buffer, value)?;
        Ok(buffer)
    }

    /// Serialize a value to a writer using the configured options.
    #[cfg(feature = "serde")]
    pub fn to_writer<W, T>(self, writer: W, value: &T) -> Result<()>
    where
        W: std::io::Write,
        T: serde::Serialize,
    {
        let serializer = self.build(writer);
        value.serialize(serializer)
    }

    /// Serialize a UbjsonValue to a Vec<u8> using the configured options.
    pub fn value_to_vec(self, value: &UbjsonValue) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.value_to_writer(&mut buffer, value)?;
        Ok(buffer)
    }

    /// Serialize a UbjsonValue to a writer using the configured options.
    pub fn value_to_writer<W>(self, writer: W, value: &UbjsonValue) -> Result<()>
    where
        W: std::io::Write,
    {
        let mut serializer = self.build(writer);
        serializer.serialize_value(value)
    }
}

/// Builder for configuring UBJSON deserialization options.
#[derive(Debug, Clone)]
pub struct DeserializerBuilder {
    max_depth: usize,
    max_size: usize,
}

impl Default for DeserializerBuilder {
    fn default() -> Self {
        Self {
            max_depth: 1000,
            max_size: 1_000_000,
        }
    }
}

impl DeserializerBuilder {
    /// Create a new deserializer builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum nesting depth to prevent stack overflow.
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Set the maximum container size to prevent DoS attacks.
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    /// Build a deserializer with the configured options for the given reader.
    pub fn build<R: std::io::Read>(self, reader: R) -> UbjsonDeserializer<R> {
        UbjsonDeserializer::with_limits(reader, self.max_depth, self.max_size)
    }

    /// Deserialize a value from a byte slice using the configured options.
    #[cfg(feature = "serde")]
    pub fn from_slice<'a, T>(self, slice: &'a [u8]) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.from_reader(slice)
    }

    /// Deserialize a value from a reader using the configured options.
    #[cfg(feature = "serde")]
    pub fn from_reader<R, T>(self, reader: R) -> Result<T>
    where
        R: std::io::Read,
        T: serde::de::DeserializeOwned,
    {
        let deserializer = self.build(reader);
        T::deserialize(deserializer)
    }

    /// Deserialize a UbjsonValue from a byte slice using the configured options.
    pub fn value_from_slice(self, slice: &[u8]) -> Result<UbjsonValue> {
        self.value_from_reader(slice)
    }

    /// Deserialize a UbjsonValue from a reader using the configured options.
    pub fn value_from_reader<R>(self, reader: R) -> Result<UbjsonValue>
    where
        R: std::io::Read,
    {
        let mut deserializer = self.build(reader);
        deserializer.deserialize_value()
    }
}