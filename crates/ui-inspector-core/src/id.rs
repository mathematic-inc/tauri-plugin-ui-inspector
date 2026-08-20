//! Opaque, time-sortable UI reference identifiers.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ulid::Ulid;

use crate::error::ReferenceIdError;

const PREFIX: &str = "ui_";

/// An opaque `ui_<ULID>` reference identifier.
///
/// The optional human-facing `@` prefix is accepted while parsing but is not
/// stored or written to filesystem paths.
///
/// # Examples
///
/// ```
/// use tauri_ui_inspector_core::ReferenceId;
///
/// let id = ReferenceId::parse("@ui_01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
/// assert_eq!(id.to_string(), "ui_01ARZ3NDEKTSV4RRFFQ69G5FAV");
/// ```
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ReferenceId(String);

impl ReferenceId {
    /// Generates a new time-sortable identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_ui_inspector_core::ReferenceId;
    ///
    /// assert!(ReferenceId::new().as_str().starts_with("ui_"));
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self(format!("{PREFIX}{}", Ulid::generate()))
    }

    /// Parses an identifier with or without a leading `@`.
    ///
    /// # Errors
    ///
    /// Returns [`ReferenceIdError`] when the value is not `ui_` followed by a
    /// valid ULID.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_ui_inspector_core::ReferenceId;
    ///
    /// let id = ReferenceId::parse("ui_01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
    /// assert_eq!(id.as_str(), "ui_01ARZ3NDEKTSV4RRFFQ69G5FAV");
    /// ```
    pub fn parse(value: impl AsRef<str>) -> Result<Self, ReferenceIdError> {
        let raw = value.as_ref().strip_prefix('@').unwrap_or(value.as_ref());
        let Some(ulid) = raw.strip_prefix(PREFIX) else {
            return Err(ReferenceIdError::new(value.as_ref()));
        };
        Ulid::from_str(ulid).map_err(|_| ReferenceIdError::new(value.as_ref()))?;
        Ok(Self(raw.to_owned()))
    }

    /// Returns the normalized identifier without a leading `@`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_ui_inspector_core::ReferenceId;
    ///
    /// let id = ReferenceId::parse("@ui_01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
    /// assert_eq!(id.as_str(), "ui_01ARZ3NDEKTSV4RRFFQ69G5FAV");
    /// ```
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the canonical human-facing `@ui_<ULID>` notation.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_ui_inspector_core::ReferenceId;
    ///
    /// let id = ReferenceId::parse("ui_01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
    /// assert_eq!(id.mention(), "@ui_01ARZ3NDEKTSV4RRFFQ69G5FAV");
    /// ```
    #[must_use]
    pub fn mention(&self) -> String {
        format!("@{}", self.0)
    }
}

impl Default for ReferenceId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ReferenceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("ReferenceId").field(&self.0).finish()
    }
}

impl fmt::Display for ReferenceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for ReferenceId {
    type Err = ReferenceIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for ReferenceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ReferenceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}
