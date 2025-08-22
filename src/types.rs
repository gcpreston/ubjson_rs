//! UBJSON type markers and constants.

use crate::error::{UbjsonError, Result};

/// UBJSON type markers as defined in the specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum UbjsonType {
    /// Null value marker 'Z'
    Null = b'Z',
    /// No-op marker 'N' (used for padding)
    NoOp = b'N',
    /// Boolean true marker 'T'
    True = b'T',
    /// Boolean false marker 'F'
    False = b'F',
    /// Signed 8-bit integer marker 'i'
    Int8 = b'i',
    /// Unsigned 8-bit integer marker 'U'
    UInt8 = b'U',
    /// Signed 16-bit integer marker 'I'
    Int16 = b'I',
    /// Signed 32-bit integer marker 'l'
    Int32 = b'l',
    /// Signed 64-bit integer marker 'L'
    Int64 = b'L',
    /// 32-bit floating point marker 'd'
    Float32 = b'd',
    /// 64-bit floating point marker 'D'
    Float64 = b'D',
    /// High-precision number marker 'H'
    HighPrecision = b'H',
    /// Character marker 'C'
    Char = b'C',
    /// String marker 'S'
    String = b'S',
    /// Array start marker '['
    ArrayStart = b'[',
    /// Array end marker ']'
    ArrayEnd = b']',
    /// Object start marker '{'
    ObjectStart = b'{',
    /// Object end marker '}'
    ObjectEnd = b'}',
}

impl UbjsonType {
    /// Convert a byte to a UbjsonType.
    pub fn from_byte(byte: u8) -> Result<Self> {
        match byte {
            b'Z' => Ok(UbjsonType::Null),
            b'N' => Ok(UbjsonType::NoOp),
            b'T' => Ok(UbjsonType::True),
            b'F' => Ok(UbjsonType::False),
            b'i' => Ok(UbjsonType::Int8),
            b'U' => Ok(UbjsonType::UInt8),
            b'I' => Ok(UbjsonType::Int16),
            b'l' => Ok(UbjsonType::Int32),
            b'L' => Ok(UbjsonType::Int64),
            b'd' => Ok(UbjsonType::Float32),
            b'D' => Ok(UbjsonType::Float64),
            b'H' => Ok(UbjsonType::HighPrecision),
            b'C' => Ok(UbjsonType::Char),
            b'S' => Ok(UbjsonType::String),
            b'[' => Ok(UbjsonType::ArrayStart),
            b']' => Ok(UbjsonType::ArrayEnd),
            b'{' => Ok(UbjsonType::ObjectStart),
            b'}' => Ok(UbjsonType::ObjectEnd),
            _ => Err(UbjsonError::InvalidTypeMarker(byte)),
        }
    }

    /// Convert the UbjsonType to its byte representation.
    pub fn to_byte(self) -> u8 {
        self as u8
    }

    /// Check if this type represents a primitive value (not a container).
    pub fn is_primitive(self) -> bool {
        match self {
            UbjsonType::Null
            | UbjsonType::True
            | UbjsonType::False
            | UbjsonType::Int8
            | UbjsonType::UInt8
            | UbjsonType::Int16
            | UbjsonType::Int32
            | UbjsonType::Int64
            | UbjsonType::Float32
            | UbjsonType::Float64
            | UbjsonType::HighPrecision
            | UbjsonType::Char
            | UbjsonType::String => true,
            UbjsonType::NoOp
            | UbjsonType::ArrayStart
            | UbjsonType::ArrayEnd
            | UbjsonType::ObjectStart
            | UbjsonType::ObjectEnd => false,
        }
    }

    /// Check if this type represents a container start marker.
    pub fn is_container_start(self) -> bool {
        matches!(self, UbjsonType::ArrayStart | UbjsonType::ObjectStart)
    }

    /// Check if this type represents a container end marker.
    pub fn is_container_end(self) -> bool {
        matches!(self, UbjsonType::ArrayEnd | UbjsonType::ObjectEnd)
    }

    /// Check if this type represents a numeric value.
    pub fn is_numeric(self) -> bool {
        match self {
            UbjsonType::Int8
            | UbjsonType::UInt8
            | UbjsonType::Int16
            | UbjsonType::Int32
            | UbjsonType::Int64
            | UbjsonType::Float32
            | UbjsonType::Float64
            | UbjsonType::HighPrecision => true,
            _ => false,
        }
    }

    /// Check if this type represents an integer value.
    pub fn is_integer(self) -> bool {
        match self {
            UbjsonType::Int8
            | UbjsonType::UInt8
            | UbjsonType::Int16
            | UbjsonType::Int32
            | UbjsonType::Int64 => true,
            _ => false,
        }
    }

    /// Check if this type represents a floating-point value.
    pub fn is_float(self) -> bool {
        matches!(self, UbjsonType::Float32 | UbjsonType::Float64)
    }
}

impl std::fmt::Display for UbjsonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            UbjsonType::Null => "null",
            UbjsonType::NoOp => "no-op",
            UbjsonType::True => "true",
            UbjsonType::False => "false",
            UbjsonType::Int8 => "int8",
            UbjsonType::UInt8 => "uint8",
            UbjsonType::Int16 => "int16",
            UbjsonType::Int32 => "int32",
            UbjsonType::Int64 => "int64",
            UbjsonType::Float32 => "float32",
            UbjsonType::Float64 => "float64",
            UbjsonType::HighPrecision => "high-precision",
            UbjsonType::Char => "char",
            UbjsonType::String => "string",
            UbjsonType::ArrayStart => "array-start",
            UbjsonType::ArrayEnd => "array-end",
            UbjsonType::ObjectStart => "object-start",
            UbjsonType::ObjectEnd => "object-end",
        };
        write!(f, "{}", name)
    }
}

/// Container optimization markers
pub mod optimization {
    /// Type marker '$' - indicates strongly-typed container
    pub const TYPE_MARKER: u8 = b'$';
    /// Count marker '#' - indicates container with known count
    pub const COUNT_MARKER: u8 = b'#';
}

#[cfg(test)]
mod tests {
    use super::*;

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
}