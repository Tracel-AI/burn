use std::collections::HashMap;

use burn::record::{PrecisionSettings, Record};
use serde::Deserialize;

use super::adapter::DefaultAdapter;
use super::de::Deserializer;
use super::error::Error;

/// A nested map/vector of tensors.
#[derive(Debug, Clone)]
pub enum NestedValue {
    /// The default value (typically for primitives like integers)
    Default,

    /// A string value
    String(String),

    /// Floating point 32-bit value
    F32(f32),

    /// Floating point 64-bit value
    F64(f64),

    /// Signed 16-bit integer value
    I16(i16),

    /// Signed 32-bit integer value
    I32(i32),

    /// Signed 64-bit integer value
    I64(i64),

    /// Unsigned 16-bit integer value used for bf16 and f16 serialization
    U16(u16),

    /// Unsigned 64-bit integer value
    U64(u64),

    /// A map of nested values (typically used for structs)
    Map(HashMap<String, NestedValue>),

    /// A vector of nested values (typically used for vector of structs)
    Vec(Vec<NestedValue>),
}

impl NestedValue {
    pub fn get_map(&self) -> Option<&HashMap<String, NestedValue>> {
        match self {
            NestedValue::Map(map) => Some(map),
            _ => None,
        }
    }

    pub fn get_string(&self) -> Option<&str> {
        match self {
            NestedValue::String(string) => Some(string),
            _ => None,
        }
    }

    pub fn get_f32(&self) -> Option<f32> {
        match self {
            NestedValue::F32(f32) => Some(*f32),

            // Convert to f32 if the data is u16 (half precision)
            NestedValue::U16(u16) => Some(half::f16::from_bits(*u16).to_f32()),
            _ => None,
        }
    }

    pub fn get_f64(&self) -> Option<f64> {
        match self {
            NestedValue::F64(f64) => Some(*f64),
            _ => None,
        }
    }

    pub fn get_i16(&self) -> Option<i16> {
        match self {
            NestedValue::I16(i16) => Some(*i16),
            _ => None,
        }
    }

    pub fn get_i32(&self) -> Option<i32> {
        match self {
            NestedValue::I32(i32) => Some(*i32),
            _ => None,
        }
    }

    pub fn get_i64(&self) -> Option<i64> {
        match self {
            NestedValue::I64(i64) => Some(*i64),
            _ => None,
        }
    }

    pub fn get_u16(&self) -> Option<u16> {
        match self {
            NestedValue::U16(u16) => Some(*u16),

            // Convert to u16 if the data is f32 (half precision)
            NestedValue::F32(f32) => Some(half::f16::from_f32(*f32).to_bits()),
            _ => None,
        }
    }

    pub fn get_u64(&self) -> Option<u64> {
        match self {
            NestedValue::U64(u64) => Some(*u64),
            _ => None,
        }
    }

    /// Deserialize a nested value into a record type.
    ///
    /// NOTE: Deserialization is done using the default adapter (see `DefaultAdapter`).
    pub fn de_into<T, PS>(self) -> Result<T, Error>
    where
        T: Record,
        PS: PrecisionSettings,
    {
        let deserializer = Deserializer::<DefaultAdapter>::new(self);

        let item = T::Item::deserialize(deserializer)?;

        // Convert the deserialized item into a Record instance
        Ok(T::from_item::<PS>(item))
    }
}
