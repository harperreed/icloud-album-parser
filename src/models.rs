//! Data structures for iCloud album API responses.
//!
//! This module defines the data models that represent iCloud API responses,
//! including album metadata, photo information, and derivative details.
//! It handles serialization/deserialization and provides helper methods for
//! working with the sometimes inconsistent response formats from Apple's API.

use log::{log, Level};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::cell::RefCell;

/// Thread-local storage for current deserialization context
thread_local! {
    static DESERIALIZE_CONTEXT: RefCell<Vec<String>> = RefCell::new(vec![]);
}

/// Context management for deserialization
/// 
/// These helper functions allow setting and clearing context during deserialization,
/// which enhances logging by providing more information about where parsing errors occur.
mod deserialize_context {
    use super::*;

    /// Sets the current deserialization context
    pub fn push_context(context: &str) {
        DESERIALIZE_CONTEXT.with(|ctx| {
            ctx.borrow_mut().push(context.to_string());
        });
    }

    /// Clears the current deserialization context
    pub fn pop_context() {
        DESERIALIZE_CONTEXT.with(|ctx| {
            let _ = ctx.borrow_mut().pop();
        });
    }

    /// Returns the current deserialization context as a string
    pub fn get_context() -> String {
        DESERIALIZE_CONTEXT.with(|ctx| {
            let contexts = ctx.borrow();
            if contexts.is_empty() {
                "unknown field".to_string()
            } else {
                contexts.join(" > ")
            }
        })
    }

    /// Logs a message with the current deserialization context
    pub fn log_with_context(level: Level, message: &str) {
        let context = get_context();
        log!(level, "[Context: {}] {}", context, message);
    }
}

/// Helper module for deserializing/serializing fields that can be either strings or numbers
/// iCloud API sometimes returns numbers as strings, so we need to handle both cases
mod string_or_number {
    
    use super::deserialize_context;
    use super::Level;
    use log::{debug, trace};
    use serde::de::{self, Visitor};
    use serde::{Deserializer, Serializer};
    use std::fmt;

    // Deserialize from either a string or number
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Push the field type to the context stack for better error messages
        deserialize_context::push_context("u64/string field");
        // Make sure we pop the context even if there's an error
        let result = deserialize_impl(deserializer);
        deserialize_context::pop_context();
        result
    }
    
    // Implementation separated to ensure context is always popped
    fn deserialize_impl<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Define a visitor that can handle both strings and numbers
        struct StringOrNumberVisitor;

        impl Visitor<'_> for StringOrNumberVisitor {
            type Value = Option<u64>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or number")
            }

            // Handle an actual number
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Some(value))
            }
            
            // Handle an i64 (smaller numbers)
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value < 0 {
                    return Ok(None);
                }
                Ok(Some(value as u64))
            }

            // Handle a string that contains a number
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value.parse::<u64>() {
                    Ok(num) => Ok(Some(num)),
                    Err(e) => {
                        // Log the error with detailed context and return None instead of failing
                        deserialize_context::log_with_context(
                            Level::Warn,
                            &format!(
                                "Type inconsistency: Failed to parse string '{}' as u64: {}. \
                                This may indicate a change in API format. \
                                Using None as fallback, but this could lead to loss of data.",
                                value, e
                            )
                        );
                        trace!("Parse error details: {:?}", e);
                        Ok(None)
                    }
                }
            }
            
            // Handle null values
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }
            
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }
        }

        deserializer.deserialize_any(StringOrNumberVisitor)
    }
    
    // Serialize back to a number (or null for None)
    pub fn serialize<S>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) => serializer.serialize_u64(*v),
            None => serializer.serialize_none(),
        }
    }
}

// Helper module for deserializing u32 values that can be strings or numbers
mod string_or_u32 {
    
    use super::deserialize_context;
    use super::Level;
    use log::{debug, trace};
    use serde::de::{self, Visitor};
    use serde::{Deserializer, Serializer};
    use std::fmt;

    // Deserialize from either a string or number
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Push the field type to the context stack for better error messages
        deserialize_context::push_context("u32/string field");
        // Make sure we pop the context even if there's an error
        let result = deserialize_impl(deserializer);
        deserialize_context::pop_context();
        result
    }
    
    // Implementation separated to ensure context is always popped
    fn deserialize_impl<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Define a visitor that can handle both strings and numbers
        struct StringOrNumberVisitor;

        impl Visitor<'_> for StringOrNumberVisitor {
            type Value = Option<u32>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or number")
            }

            // Handle an actual number
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value > u32::MAX as u64 {
                    return Ok(None);
                }
                Ok(Some(value as u32))
            }
            
            // Handle an i64 (smaller numbers)
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value < 0 || value > u32::MAX as i64 {
                    return Ok(None);
                }
                Ok(Some(value as u32))
            }

            // Handle a string that contains a number
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value.parse::<u32>() {
                    Ok(num) => Ok(Some(num)),
                    Err(e) => {
                        // Log the error with detailed context and return None instead of failing
                        deserialize_context::log_with_context(
                            Level::Warn,
                            &format!(
                                "Type inconsistency: Failed to parse string '{}' as u32: {}. \
                                This may indicate a change in API format. \
                                Field will be treated as null, which may affect application behavior.",
                                value, e
                            )
                        );
                        debug!("Field parsing context: Type expected was u32, received string '{}'", value);
                        trace!("Parse error details: {:?}", e);
                        Ok(None)
                    }
                }
            }
            
            // Handle null values
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }
            
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }
        }

        deserializer.deserialize_any(StringOrNumberVisitor)
    }
    
    // Serialize back to a number (or null for None)
    pub fn serialize<S>(value: &Option<u32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) => serializer.serialize_u32(*v),
            None => serializer.serialize_none(),
        }
    }
}

/// Represents a derivative (variant) of an image with different sizing/quality
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Derivative {
    /// Checksum identifier for the derivative
    pub checksum: String,
    /// File size in bytes - can be either a number or a string in the API
    #[serde(rename = "fileSize")]
    #[serde(default)]
    #[serde(with = "string_or_number")]
    pub file_size: Option<u64>,
    /// Width of the image in pixels
    #[serde(default)]
    #[serde(with = "string_or_u32")]
    pub width: Option<u32>,
    /// Height of the image in pixels
    #[serde(default)]
    #[serde(with = "string_or_u32")]
    pub height: Option<u32>,
    /// URL to download the image (populated later in the process)
    pub url: Option<String>,
}

/// Represents an image in the iCloud shared album
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Image {
    /// Unique identifier for the photo
    #[serde(rename = "photoGuid")]
    pub photo_guid: String,
    /// Map of derivative identifiers to their details
    pub derivatives: HashMap<String, Derivative>,
    /// Optional caption for the image
    pub caption: Option<String>,
    /// Creation date of the image
    #[serde(rename = "dateCreated")]
    pub date_created: Option<String>,
    /// Batch creation date
    #[serde(rename = "batchDateCreated")]
    pub batch_date_created: Option<String>,
    /// Width of the original image in pixels
    #[serde(default)]
    #[serde(with = "string_or_u32")]
    pub width: Option<u32>,
    /// Height of the original image in pixels
    #[serde(default)]
    #[serde(with = "string_or_u32")]
    pub height: Option<u32>,
}

/// Metadata about the iCloud shared album
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    /// Name of the shared album
    #[serde(rename = "streamName")]
    pub stream_name: String,
    /// First name of the album owner
    #[serde(rename = "userFirstName")]
    pub user_first_name: String,
    /// Last name of the album owner
    #[serde(rename = "userLastName")]
    pub user_last_name: String,
    /// Stream change tag for tracking updates
    #[serde(rename = "streamCtag")]
    pub stream_ctag: String,
    /// Number of items returned in this response
    #[serde(rename = "itemsReturned")]
    pub items_returned: u32,
    /// Location information for photos in the album
    pub locations: serde_json::Value,
}

/// Raw API response from the iCloud webstream endpoint
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponse {
    /// List of photos in the album
    pub photos: Vec<Image>,
    /// List of photo GUIDs in the album
    #[serde(rename = "photoGuids")]
    pub photo_guids: Vec<String>,
    /// Name of the shared album
    #[serde(rename = "streamName")]
    pub stream_name: Option<String>,
    /// First name of the album owner
    #[serde(rename = "userFirstName")]
    pub user_first_name: Option<String>,
    /// Last name of the album owner
    #[serde(rename = "userLastName")]
    pub user_last_name: Option<String>,
    /// Stream change tag for tracking updates
    #[serde(rename = "streamCtag")]
    pub stream_ctag: Option<String>,
    /// Number of items returned in this response
    #[serde(rename = "itemsReturned")]
    pub items_returned: Option<String>,
    /// Location information for photos in the album
    pub locations: Option<serde_json::Value>,
}

/// Final response with processed photos and metadata
#[derive(Debug, Clone)]
pub struct ICloudResponse {
    /// Metadata about the album
    pub metadata: Metadata,
    /// Processed photos with URLs populated
    pub photos: Vec<Image>,
}