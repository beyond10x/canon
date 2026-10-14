//! Keys written with no value: a key left out takes its default, or is refused when it has none; a
//! key written with no value is an error.
//!
//! YAML reads `key:`, `key: ~` and `key: null` as null. Serde would read each as an absent
//! optional field, so an author who wrote a key and forgot its value would silently get the
//! default. Every optional field, every defaulted section, every required field and every
//! identifier in the model goes through one of these functions instead, and the refusal of a null
//! says what the author can do: leave out a key that has a default, give a value to one that is
//! required.

use serde::{Deserialize, Deserializer, de::Error};

/// The refusal of a null on a key that may be left out.
const NULL: &str = "an explicit null is not allowed here; leave the key out to take its default";

/// The refusal of a null on a required key: leaving it out is refused as `missing field`, so the
/// refusal does not suggest it.
const REQUIRED: &str = "this key is required and needs a value; an explicit null is not one";

/// The refusal of a null on the key of a written form (a predicate's `all`, a property subject's
/// `action`): exactly one form is written, so leaving it out is refused too.
const FORM: &str = "this key needs a value; an explicit null is not one";

/// The refusal of a null where an identifier is written: a list item or a map key.
const IDENTIFIER: &str = "an identifier is required here; an explicit null is not one";

fn not_null<'de, D, T>(deserializer: D, why: &'static str) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)?.ok_or_else(|| D::Error::custom(why))
}

/// For a field declared `#[serde(default, deserialize_with = "present::optional")]`.
pub(crate) fn optional<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    not_null(deserializer, NULL).map(Some)
}

/// For the key of one form among several, declared
/// `#[serde(default, deserialize_with = "present::form")]`: absent when left out, refused with
/// [`FORM`] when written as null.
pub(crate) fn form<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    not_null(deserializer, FORM).map(Some)
}

/// For a field declared `#[serde(default, deserialize_with = "present::defaulted")]`: a section
/// that takes its default when left out.
pub(crate) fn defaulted<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    not_null(deserializer, NULL)
}

/// For a required field, declared `#[serde(deserialize_with = "present::required")]`. Every
/// required field is declared so, identifiers included: serde reads a left-out field of a type it
/// has no `deserialize_with` for by asking that type to read a null, which names nothing; for a
/// field with `deserialize_with`, serde refuses the left-out key itself, naming it
/// (`` missing field `id` ``). A null written on the key is refused with [`REQUIRED`].
pub(crate) fn required<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    not_null(deserializer, REQUIRED)
}

/// For every identifier, read as text: a null is refused with [`IDENTIFIER`] rather than read as
/// the text `~` or `null`.
pub(crate) fn identifier<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    not_null(deserializer, IDENTIFIER)
}

/// A value that must be written: deserializes `T`, refusing an explicit null or an empty value
/// with [`REQUIRED`]. Used for declaration bodies and map values, where `name:` with nothing after
/// it is a key written with no value.
pub(crate) struct Required<T>(pub(crate) T);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Required<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        required(deserializer).map(Required)
    }
}

/// For a JSON-valued field declared `#[serde(default, deserialize_with = "present::optional_json")]`:
/// [`optional`], reading the YAML value as the JSON value it writes. YAML says more than JSON can,
/// and serde would quietly read a non-finite number (`.nan`, `.inf`) as JSON `null`; here a
/// non-finite number, a mapping key that is not a string and a tagged value are refused instead.
pub(crate) fn optional_json<'de, D>(deserializer: D) -> Result<Option<serde_json::Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let yaml: serde_yaml_ng::Value = not_null(deserializer, NULL)?;
    json(yaml).map(Some).map_err(D::Error::custom)
}

/// The JSON value a YAML value writes, or why it has none.
fn json(yaml: serde_yaml_ng::Value) -> Result<serde_json::Value, &'static str> {
    use serde_json::Value as Json;
    use serde_yaml_ng::Value as Yaml;
    Ok(match yaml {
        Yaml::Null => Json::Null,
        Yaml::Bool(value) => Json::Bool(value),
        Yaml::Number(number) => {
            if let Some(value) = number.as_u64() {
                Json::from(value)
            } else if let Some(value) = number.as_i64() {
                Json::from(value)
            } else {
                number
                    .as_f64()
                    .and_then(serde_json::Number::from_f64)
                    .map(Json::Number)
                    .ok_or("a number that is not finite has no JSON value")?
            }
        }
        Yaml::String(text) => Json::String(text),
        Yaml::Sequence(items) => {
            Json::Array(items.into_iter().map(json).collect::<Result<_, _>>()?)
        }
        Yaml::Mapping(mapping) => {
            let mut object = serde_json::Map::new();
            for (key, value) in mapping {
                let Yaml::String(key) = key else {
                    return Err("a mapping key that is not a string has no JSON value");
                };
                object.insert(key, json(value)?);
            }
            Json::Object(object)
        }
        Yaml::Tagged(_) => return Err("a tagged YAML value has no JSON value"),
    })
}
