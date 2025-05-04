//! ABOUTME: This module contains the data models for the iCloud album parser.
//! ABOUTME: It defines structures corresponding to the iCloud API response format.

use serde::{Deserialize, Serialize, Deserializer};
use std::collections::HashMap;

/// Helper module for deserializing fields that can be either strings or numbers
/// iCloud API sometimes returns numbers as strings, so we need to handle both cases
mod string_or_number {
    use super::*;
    use serde::de::{self, Visitor};
    use std::fmt;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Define a visitor that can handle both strings and numbers
        struct StringOrNumberVisitor;

        impl<'de> Visitor<'de> for StringOrNumberVisitor {
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
                    Err(_) => {
                        // Just log the error and return None instead of failing
                        eprintln!("Failed to parse string as number: {}", value);
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
    pub width: Option<u32>,
    /// Height of the image in pixels
    #[serde(default)]
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
    pub width: Option<u32>,
    /// Height of the original image in pixels
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