//! ABOUTME: This module handles API calls to the iCloud shared album API.
//! ABOUTME: It implements functions to fetch metadata and photo information.

use crate::models::{Image, Metadata};
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
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

/// Fetches URLs for photo assets from the iCloud API
///
/// This function makes a POST request to the webasseturls endpoint with an array of photo GUIDs
/// and returns a map of GUID to URL for each asset.
///
/// # Arguments
///
/// * `client` - A reqwest HTTP client
/// * `base_url` - The base URL for API requests
/// * `photo_guids` - A slice of photo GUIDs to fetch URLs for
///
/// # Returns
///
/// A HashMap mapping from photo GUID to its full URL
pub async fn get_asset_urls(
    client: &Client,
    base_url: &str,
    photo_guids: &[String],
) -> Result<HashMap<String, String>, Box<dyn Error>> {
    // Early exit if there are no photo GUIDs
    if photo_guids.is_empty() {
        return Ok(HashMap::new());
    }
    
    // Build the URL for the webasseturls endpoint
    let url = format!("{}webasseturls", base_url);
    
    // Create the payload with the photo GUIDs
    let payload = json!({ "photoGuids": photo_guids });

    // Make the POST request with retry logic
    let mut retries = 0;
    let max_retries = 3;
    let mut last_error = None;
    
    while retries < max_retries {
        // Make the POST request
        match client.post(&url).json(&payload).send().await {
            Ok(resp) => {
                // Check if the request was successful
                if resp.status().is_success() {
                    // Parse the response as JSON
                    match resp.json::<serde_json::Value>().await {
                        Ok(data) => {
                            // Get the items object from the response
                            let items_val = &data["items"];
                            let mut results = HashMap::new();

                            // Extract the URL for each photo GUID
                            if let Some(obj) = items_val.as_object() {
                                for (guid, value) in obj.iter() {
                                    let url_location = value["url_location"].as_str().unwrap_or("");
                                    let url_path = value["url_path"].as_str().unwrap_or("");
                                    let full_url = format!("https://{}{}", url_location, url_path);
                                    results.insert(guid.to_string(), full_url);
                                }
                            }

                            return Ok(results);
                        },
                        Err(e) => {
                            last_error = Some(format!("Failed to parse webasseturls response: {}", e).into());
                            retries += 1;
                            tokio::time::sleep(tokio::time::Duration::from_millis(500 * retries)).await;
                            continue;
                        }
                    }
                } else if resp.status().as_u16() == 400 {
                    // For 400 Bad Request, we'll try a different approach
                    // Apple sometimes rejects batch requests, so try to get the checksums instead
                    eprintln!("Warning: webasseturls request failed with 400 Bad Request. The API may be rejecting batch requests.");
                    eprintln!("Returning empty map to continue with partial functionality.");
                    
                    // Instead of failing, return an empty map
                    // This will allow partial functionality - photos won't have URLs but metadata will still work
                    return Ok(HashMap::new());
                } else {
                    last_error = Some(format!("webasseturls request failed with status {}", resp.status()).into());
                    retries += 1;
                    tokio::time::sleep(tokio::time::Duration::from_millis(500 * retries)).await;
                    continue;
                }
            },
            Err(e) => {
                last_error = Some(format!("webasseturls request error: {}", e).into());
                retries += 1;
                tokio::time::sleep(tokio::time::Duration::from_millis(500 * retries)).await;
                continue;
            }
        }
    }
    
    // If we get here, all retries failed
    Err(last_error.unwrap_or_else(|| "webasseturls request failed after retries".into()))
}