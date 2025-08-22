//! Error types for UBJSON operations.

use std::fmt;

/// Errors that can occur during UBJSON serialization and deserialization.
#[derive(Debug, thiserror::Error)]
pub enum UbjsonError {
    /// I/O error occurred during reading or writing.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid UBJSON format encountered.
    #[error("Invalid UBJSON format: {0}")]
    InvalidFormat(String),

    /// Unexpected end of input while parsing.
    #[error("Unexpected end of input")]
    UnexpectedEof,

    /// Invalid UTF-8 sequence in string data.
    #[error("Invalid UTF-8 sequence: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),

    /// Container size limit exceeded to prevent DoS attacks.
    #[error("Container size limit exceeded: {0}")]
    SizeLimitExceeded(usize),

    /// Nesting depth limit exceeded to prevent stack overflow.
    #[error("Nesting depth limit exceeded: {0}")]
    DepthLimitExceeded(usize),

    /// Unsupported type encountered.
    #[error("Unsupported type: {0}")]
    UnsupportedType(String),

    /// Error from serde serialization/deserialization.
    #[error("Serde error: {0}")]
    Serde(String),

    /// Invalid type marker encountered.
    #[error("Invalid type marker: {0:#x}")]
    InvalidTypeMarker(u8),

    /// Container length mismatch.
    #[error("Container length mismatch: expected {expected}, found {actual}")]
    LengthMismatch { expected: usize, actual: usize },

    /// Invalid high-precision number format.
    #[error("Invalid high-precision number: {0}")]
    InvalidHighPrecision(String),

    /// Invalid character value.
    #[error("Invalid character value: {0}")]
    InvalidChar(String),
}

impl UbjsonError {
    /// Create a new InvalidFormat error with a custom message.
    pub fn invalid_format<T: fmt::Display>(msg: T) -> Self {
        UbjsonError::InvalidFormat(msg.to_string())
    }

    /// Create a new UnsupportedType error with a custom message.
    pub fn unsupported_type<T: fmt::Display>(msg: T) -> Self {
        UbjsonError::UnsupportedType(msg.to_string())
    }

    /// Create a new Serde error with a custom message.
    pub fn serde<T: fmt::Display>(msg: T) -> Self {
        UbjsonError::Serde(msg.to_string())
    }
}

/// Result type alias for UBJSON operations.
pub type Result<T> = std::result::Result<T, UbjsonError>;

#[cfg(feature = "serde")]
impl serde::ser::Error for UbjsonError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        UbjsonError::serde(msg)
    }
}

#[cfg(feature = "serde")]
impl serde::de::Error for UbjsonError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        UbjsonError::serde(msg)
    }
}