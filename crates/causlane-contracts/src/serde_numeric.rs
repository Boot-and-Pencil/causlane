//! Shared serde numeric guards for document-boundary DTOs.
//!
//! `noyalib` represents YAML numbers as signed integers or floats. These
//! helpers reject floating-point values for unsigned integer DTO fields so a
//! large YAML integer cannot be accepted after lossy float rounding. JSON
//! deserializers that provide a real `u64` remain accepted.

use std::fmt;

use serde::de::{Error, Unexpected, Visitor};

/// Deserialize a `u64` while rejecting lossy float-backed values.
///
/// # Errors
/// Returns a serde error when the input is negative, non-integer or only
/// available as a floating-point value.
pub fn deserialize_u64_lossless<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_any(LosslessU64Visitor)
}

/// Deserialize an optional `u64` while rejecting lossy float-backed values.
///
/// # Errors
/// Returns a serde error when a present value is negative, non-integer or only
/// available as a floating-point value.
pub fn deserialize_option_u64_lossless<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_option(LosslessOptionU64Visitor)
}

struct LosslessU64Visitor;

impl Visitor<'_> for LosslessU64Visitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a lossless unsigned integer")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(value)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        u64::try_from(value)
            .map_err(|_| E::invalid_value(Unexpected::Signed(value), &"a non-negative integer"))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Err(E::invalid_value(
            Unexpected::Float(value),
            &"a lossless unsigned integer, not a float-backed YAML number",
        ))
    }
}

struct LosslessOptionU64Visitor;

impl<'de> Visitor<'de> for LosslessOptionU64Visitor {
    type Value = Option<u64>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an optional lossless unsigned integer")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(None)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_u64_lossless(deserializer).map(Some)
    }
}
