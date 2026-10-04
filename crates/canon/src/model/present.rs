//! Optional fields: a key left out takes its default; a key written with no value is an error.
//!
//! YAML reads `key:`, `key: ~` and `key: null` as null. Serde would read each as an absent
//! optional field, so an author who wrote a key and forgot its value would silently get the
//! default. Every optional field and every defaulted section in the model goes through one of these
//! two functions instead.

use serde::{Deserialize, Deserializer, de::Error};

const NULL: &str = "an explicit null is not allowed here; leave the key out to take its default";

/// For a field declared `#[serde(default, deserialize_with = "present::optional")]`.
pub(crate) fn optional<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    required(deserializer).map(Some)
}

/// For a field declared `#[serde(default, deserialize_with = "present::required")]`, a required
/// field declared `#[serde(deserialize_with = "present::required")]`, and every identifier.
pub(crate) fn required<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)?.ok_or_else(|| D::Error::custom(NULL))
}

/// A value that must be written: deserializes `T`, refusing an explicit null or an empty value.
/// Used for declaration bodies, where `name:` with nothing after it is a key written with no value.
pub(crate) struct Required<T>(pub(crate) T);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Required<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        required(deserializer).map(Required)
    }
}
