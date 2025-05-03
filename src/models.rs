//! ABOUTME: This module contains the data models for the iCloud album parser.
//! ABOUTME: It defines structures corresponding to the iCloud API response format.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a derivative (variant) of an image with different sizing/quality
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Derivative {
    /// Checksum identifier for the derivative
    pub checksum: String,
    /// File size in bytes
    #[serde(rename = "fileSize")]
    pub file_size: Option<u64>,
    /// Width of the image in pixels
    pub width: Option<u32>,
    /// Height of the image in pixels
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