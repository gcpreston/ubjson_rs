# UBJSON Library Design Document

## Overview

The UBJSON library will be implemented as a Rust crate that provides efficient serialization and deserialization of Universal Binary JSON data. The library will follow Rust best practices, integrate with serde for automatic derive support, and implement all UBJSON specification features including container optimizations.

The library will be structured around a core set of types representing UBJSON values, with separate serializer and deserializer implementations that handle the binary format conversion.

## Architecture

The library follows a layered architecture:

```
┌─────────────────────────────────────┐
│           Public API Layer          │
│  (serialize/deserialize functions)  │
├─────────────────────────────────────┤
│         Serde Integration           │
│    (Serializer/Deserializer)        │
├─────────────────────────────────────┤
│          Core Value Types           │
│     (UbjsonValue enum)              │
├─────────────────────────────────────┤
│         Binary I/O Layer            │
│    (Reader/Writer traits)           │
├─────────────────────────────────────┤
│        Low-level Encoding           │
│   (type markers, length encoding)   │
└─────────────────────────────────────┘
```

## Components and Interfaces

### Core Value Type

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum UbjsonValue {
    Null,
    Bool(bool),
    Int8(i8),
    UInt8(u8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    HighPrecision(String),
    Char(char),
    String(String),
    Array(Vec<UbjsonValue>),
    Object(std::collections::HashMap<String, UbjsonValue>),
    // Optimized containers
    StronglyTypedArray {
        element_type: UbjsonType,
        count: Option<usize>,
        elements: Vec<UbjsonValue>,
    },
    StronglyTypedObject {
        value_type: UbjsonType,
        count: Option<usize>,
        pairs: std::collections::HashMap<String, UbjsonValue>,
    },
}
```

### Type Markers

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UbjsonType {
    Null = b'Z' as isize,
    NoOp = b'N' as isize,
    True = b'T' as isize,
    False = b'F' as isize,
    Int8 = b'i' as isize,
    UInt8 = b'U' as isize,
    Int16 = b'I' as isize,
    Int32 = b'l' as isize,
    Int64 = b'L' as isize,
    Float32 = b'd' as isize,
    Float64 = b'D' as isize,
    HighPrecision = b'H' as isize,
    Char = b'C' as isize,
    String = b'S' as isize,
    ArrayStart = b'[' as isize,
    ArrayEnd = b']' as isize,
    ObjectStart = b'{' as isize,
    ObjectEnd = b'}' as isize,
}
```

### Serializer Interface

```rust
pub struct UbjsonSerializer<W: Write> {
    writer: W,
    optimize_containers: bool,
}

impl<W: Write> UbjsonSerializer<W> {
    pub fn new(writer: W) -> Self;
    pub fn with_optimization(writer: W, optimize: bool) -> Self;
    pub fn serialize_value(&mut self, value: &UbjsonValue) -> Result<(), UbjsonError>;
}
```

### Deserializer Interface

```rust
pub struct UbjsonDeserializer<R: Read> {
    reader: R,
    max_depth: usize,
    max_size: usize,
}

impl<R: Read> UbjsonDeserializer<R> {
    pub fn new(reader: R) -> Self;
    pub fn with_limits(reader: R, max_depth: usize, max_size: usize) -> Self;
    pub fn deserialize_value(&mut self) -> Result<UbjsonValue, UbjsonError>;
}
```

### Public API Functions

```rust
// High-level convenience functions
pub fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>, UbjsonError>;
pub fn to_writer<W: Write, T: Serialize>(writer: W, value: &T) -> Result<(), UbjsonError>;
pub fn from_slice<'a, T: Deserialize<'a>>(slice: &'a [u8]) -> Result<T, UbjsonError>;
pub fn from_reader<R: Read, T: DeserializeOwned>(reader: R) -> Result<T, UbjsonError>;

// Value-based functions
pub fn value_to_vec(value: &UbjsonValue) -> Result<Vec<u8>, UbjsonError>;
pub fn value_from_slice(slice: &[u8]) -> Result<UbjsonValue, UbjsonError>;
```

## Data Models

### Container Optimization Logic

The library will implement container optimization by analyzing collections during serialization:

1. **Strongly-Typed Arrays**: When all elements in an array are the same UBJSON type, use format: `[$][type][#][count][elements...]`
2. **Strongly-Typed Objects**: When all values in an object are the same UBJSON type, use format: `{$][type][#][count][key-value pairs...]`
3. **Count Optimization**: When container size is known, include count for more efficient parsing

### Memory Management

- Use `Cow<str>` for strings to enable zero-copy deserialization when possible
- Implement streaming deserialization to handle large datasets without loading everything into memory
- Provide options for limiting maximum container sizes to prevent DoS attacks

### Serde Integration

The library will implement `serde::Serializer` and `serde::Deserializer` traits to enable automatic serialization of Rust types:

```rust
impl<W: Write> serde::Serializer for UbjsonSerializer<W> {
    type Ok = ();
    type Error = UbjsonError;
    // ... trait implementation
}

impl<'de, R: Read> serde::Deserializer<'de> for UbjsonDeserializer<R> {
    type Error = UbjsonError;
    // ... trait implementation
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum UbjsonError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid UBJSON format: {0}")]
    InvalidFormat(String),
    
    #[error("Unexpected end of input")]
    UnexpectedEof,
    
    #[error("Invalid UTF-8 sequence: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),
    
    #[error("Container size limit exceeded: {0}")]
    SizeLimitExceeded(usize),
    
    #[error("Nesting depth limit exceeded: {0}")]
    DepthLimitExceeded(usize),
    
    #[error("Unsupported type: {0}")]
    UnsupportedType(String),
    
    #[error("Serde error: {0}")]
    Serde(String),
}
```

### Error Recovery

- Provide detailed error messages with byte positions when possible
- Implement graceful handling of malformed data
- Include context information in error messages to aid debugging

## Testing Strategy

### Unit Tests

1. **Type Encoding Tests**: Verify each UBJSON type is correctly encoded and decoded
2. **Container Tests**: Test standard and optimized container formats
3. **Edge Cases**: Test empty containers, large numbers, Unicode strings
4. **Error Conditions**: Test malformed input handling and error reporting

### Integration Tests

1. **Serde Compatibility**: Test with various Rust types using derive macros
2. **Round-trip Tests**: Ensure serialize → deserialize produces identical data
3. **Performance Tests**: Benchmark against other serialization formats
4. **Compatibility Tests**: Test against reference UBJSON implementations

### Property-Based Testing

Use `proptest` or `quickcheck` to generate random data structures and verify:
- Round-trip serialization consistency
- Container optimization correctness
- Error handling robustness

### Test Data

- Include test vectors from the UBJSON specification
- Create comprehensive test suite covering all type combinations
- Test with real-world data structures and edge cases

## Performance Considerations

### Optimization Strategies

1. **Zero-Copy Deserialization**: Use `Cow<str>` and byte slices where possible
2. **Streaming Processing**: Process data incrementally to reduce memory usage
3. **Container Pre-allocation**: Use known container sizes to pre-allocate collections
4. **Type-Specific Optimizations**: Optimize common cases like homogeneous arrays

### Memory Usage

- Implement configurable limits for container sizes and nesting depth
- Use efficient data structures (e.g., `SmallVec` for small arrays)
- Minimize allocations during serialization/deserialization

### Benchmarking

- Compare performance against JSON, MessagePack, and other binary formats
- Measure both serialization and deserialization performance
- Test with various data sizes and structures
- Profile memory usage patterns