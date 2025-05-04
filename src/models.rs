//! Data structures for iCloud album API responses.
//!
//! This module defines the data models that represent iCloud API responses,
//! including album metadata, photo information, and derivative details.
//! It handles serialization/deserialization and provides helper methods for
//! working with the sometimes inconsistent response formats from Apple's API.

use log::{log, Level};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Context type for deserialization error reporting
///
/// This struct holds the current context information in a more explicit
/// and thread-safe way without using thread-local state.
#[derive(Clone, Debug, Default)]
pub struct DeserializeContext {
    context_path: Vec<String>,
}

impl DeserializeContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            context_path: Vec::new(),
        }
    }

    /// Create a new context with a single element
    pub fn with_context(context: &str) -> Self {
        let mut ctx = Self::new();
        ctx.push(context);
        ctx
    }

    /// Add a context element to the path
    pub fn push(&mut self, context: &str) {
        self.context_path.push(context.to_string());
    }

    /// Remove the last context element from the path
    pub fn pop(&mut self) -> Option<String> {
        self.context_path.pop()
    }

    /// Get the context path
    fn path_string(&self) -> String {
        if self.context_path.is_empty() {
            "unknown field".to_string()
        } else {
            self.context_path.join(" > ")
        }
    }

    /// Create a new context by extending the current one
    pub fn extend(&self, context: &str) -> Self {
        let mut new_ctx = self.clone();
        new_ctx.push(context);
        new_ctx
    }

    /// Logs a message with the current context
    pub fn log(&self, level: Level, message: &str) {
        log!(level, "[Context: {}] {}", self, message);
    }
}

impl fmt::Display for DeserializeContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path_string())
    }
}

/// Helper module for deserializing/serializing fields that can be either strings or numbers
/// iCloud API sometimes returns numbers as strings, so we need to handle both cases
mod string_or_number {

    use super::DeserializeContext;
    use super::Level;
    use log::trace;
    use serde::de::{self, Visitor};
    use serde::{Deserializer, Serializer};
    use std::cell::RefCell;
    use std::fmt;
    use std::thread_local;

    // We'll use a private thread-local variable just for this deserializer
    // This allows us to maintain the existing API while improving the implementation
    thread_local! {
        static CURRENT_CONTEXT: RefCell<DeserializeContext> = RefCell::new(DeserializeContext::new());
    }

    // Deserialize from either a string or number
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Create a context for this specific deserialization operation
        let ctx = DeserializeContext::with_context("u64/string field");

        // Store the context in our thread_local for the duration of this call
        CURRENT_CONTEXT.with(|current_ctx| {
            *current_ctx.borrow_mut() = ctx;
        });

        let result = deserialize_impl(deserializer);

        // Clear the context when we're done
        CURRENT_CONTEXT.with(|current_ctx| {
            *current_ctx.borrow_mut() = DeserializeContext::new();
        });

        result
    }

    // Implementation for deserialization
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
                        CURRENT_CONTEXT.with(|ctx| {
                            ctx.borrow().log(
                                Level::Warn,
                                &format!(
                                    "Type inconsistency: Failed to parse string '{}' as u64: {}. \
                                    This may indicate a change in API format. \
                                    Using None as fallback, but this could lead to loss of data.",
                                    value, e
                                ),
                            );
                        });
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

    use super::DeserializeContext;
    use super::Level;
    use log::trace;
    use serde::de::{self, Visitor};
    use serde::{Deserializer, Serializer};
    use std::cell::RefCell;
    use std::fmt;
    use std::thread_local;

    // We'll use a private thread-local variable just for this deserializer
    // This allows us to maintain the existing API while improving the implementation
    thread_local! {
        static CURRENT_CONTEXT: RefCell<DeserializeContext> = RefCell::new(DeserializeContext::new());
    }

    // Deserialize from either a string or number
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Create a context for this specific deserialization operation
        let ctx = DeserializeContext::with_context("u32/string field");

        // Store the context in our thread_local for the duration of this call
        CURRENT_CONTEXT.with(|current_ctx| {
            *current_ctx.borrow_mut() = ctx;
        });

        let result = deserialize_impl(deserializer);

        // Clear the context when we're done
        CURRENT_CONTEXT.with(|current_ctx| {
            *current_ctx.borrow_mut() = DeserializeContext::new();
        });

        result
    }

    // Implementation for deserialization
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
                        CURRENT_CONTEXT.with(|ctx| {
                            ctx.borrow().log(
                                Level::Warn,
                                &format!(
                                    "Type inconsistency: Failed to parse string '{}' as u32: {}. \
                                    This may indicate a change in API format. \
                                    Field will be treated as null, which may affect application behavior.",
                                    value, e
                                )
                            );
                        });
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
