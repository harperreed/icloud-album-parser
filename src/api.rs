//! ABOUTME: This module handles API calls to the iCloud shared album API.
//! ABOUTME: It implements functions to fetch metadata and photo information.

use crate::models::{Image, Metadata};
use reqwest::Client;
use serde_json::json;
use std::error::Error;

/// Fetches metadata and photos from the iCloud API
///
/// This function makes a POST request to the webstream endpoint and extracts
/// the metadata and photos from the response.
///
/// # Arguments
///
/// * `client` - A reqwest HTTP client
/// * `base_url` - The base URL for API requests
///
/// # Returns
///
/// A tuple containing a vector of Images and Metadata information
pub async fn get_api_response(
    client: &Client,
    base_url: &str,
) -> Result<(Vec<Image>, Metadata), Box<dyn Error>> {
    // Build the URL for the webstream endpoint
    let url = format!("{}webstream", base_url);
    
    // Create the payload with a null streamCtag
    let payload = json!({ "streamCtag": null });

    // Make the POST request
    let resp = client.post(&url).json(&payload).send().await?;

    // Check if the request was successful
    if !resp.status().is_success() {
        return Err(format!("webstream request failed with status {}", resp.status()).into());
    }

    // Parse the response as JSON
    let data: serde_json::Value = resp.json().await?;
    
    // Extract the photos array from the JSON
    // Create a longer-lived empty vector to use as a fallback
    let empty_vec = Vec::new();
    let photos_raw = data["photos"].as_array().unwrap_or(&empty_vec);
    let mut photos: Vec<Image> = Vec::with_capacity(photos_raw.len());
    
    // Parse each photo into an Image struct
    for photo in photos_raw {
        match serde_json::from_value::<Image>(photo.clone()) {
            Ok(parsed) => photos.push(parsed),
            Err(e) => eprintln!("Failed to parse photo: {}", e),
        }
    }

    // Extract the metadata fields from the JSON
    let metadata = Metadata {
        stream_name: data["streamName"].as_str().unwrap_or("").to_string(),
        user_first_name: data["userFirstName"].as_str().unwrap_or("").to_string(),
        user_last_name: data["userLastName"].as_str().unwrap_or("").to_string(),
        stream_ctag: data["streamCtag"].as_str().unwrap_or("").to_string(),
        items_returned: data["itemsReturned"].as_u64().unwrap_or(0) as u32,
        locations: data["locations"].clone(),
    };

    Ok((photos, metadata))
}