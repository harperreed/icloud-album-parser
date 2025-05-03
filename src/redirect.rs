//! ABOUTME: This module handles redirect responses from the iCloud API.
//! ABOUTME: It implements logic to handle Apple's 330 status redirect for shared album requests.

use reqwest::{Client, StatusCode};
use serde_json::json;

/// Handles redirects from the iCloud API
///
/// This function makes a request to the base URL and checks if it receives a 330 redirect status code.
/// If it does, it extracts the new host from the response and builds a new base URL.
/// If not, it returns the original base URL.
///
/// # Arguments
///
/// * `client` - A reqwest HTTP client
/// * `base_url` - The original base URL
/// * `token` - The iCloud album token
///
/// # Returns
///
/// A string containing either the original base URL or a redirected URL
pub async fn get_redirected_base_url(
    client: &Client, 
    base_url: &str, 
    token: &str
) -> Result<String, Box<dyn std::error::Error>> {
    // Build the URL for the webstream endpoint
    let url = format!("{}webstream", base_url);
    
    // Create the payload with a null streamCtag
    let payload = json!({ "streamCtag": null });

    // Make the POST request
    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await?;

    // Check if we got a 330 status code (Apple's redirect)
    if resp.status() == StatusCode::from_u16(330).unwrap() {
        // Parse the response body as JSON
        let body: serde_json::Value = resp.json().await?;
        
        // Look for the X-Apple-MMe-Host field
        if let Some(host_val) = body["X-Apple-MMe-Host"].as_str() {
            // Build and return the new base URL
            return Ok(format!("https://{}/{}/sharedstreams/", host_val, token));
        }
    }

    // If we didn't get a redirect or couldn't parse the host, return the original URL
    Ok(base_url.to_string())
}

// All testing is done in the separate integration tests