//! ABOUTME: This library provides functionality to interact with iCloud shared albums.
//! ABOUTME: It allows fetching metadata and photos from an album using a shared token.

/// Module containing data model structures
pub mod models;

/// Module handling the base URL generation for API calls
pub mod base_url;

/// Module handling redirects from the iCloud API
pub mod redirect;

/// Module for API calls to fetch metadata and photos
pub mod api;

/// Module for enriching photos with their URLs
pub mod enrich;

/// Main entry point for fetching photos from an iCloud shared album
///
/// This function orchestrates the entire process of:
/// 1. Generating the base URL from the token
/// 2. Handling any redirects
/// 3. Fetching the album metadata and photos
/// 4. Fetching the URLs for all photos
/// 5. Enriching the photos with their URLs
///
/// # Arguments
///
/// * `token` - The iCloud shared album token
///
/// # Returns
///
/// A Result containing an ICloudResponse with metadata and photos on success, or an error on failure
pub async fn get_icloud_photos(token: &str) -> Result<models::ICloudResponse, Box<dyn std::error::Error>> {
    // Create a reqwest client
    let client = reqwest::Client::new();

    // 1. Compute the base URL from the token
    let base_url = base_url::get_base_url(token);

    // 2. Handle any redirects
    let redirected_url = redirect::get_redirected_base_url(&client, &base_url, token).await?;

    // 3. Fetch the metadata and photos
    let (mut photos, metadata) = api::get_api_response(&client, &redirected_url).await?;

    // 4. Extract all photo GUIDs
    let photo_guids: Vec<String> = photos.iter().map(|p| p.photo_guid.clone()).collect();

    // 5. Fetch the URLs for all photos
    let all_urls = api::get_asset_urls(&client, &redirected_url, &photo_guids).await?;

    // 6. Enrich the photos with their URLs
    enrich::enrich_photos_with_urls(&mut photos, &all_urls);

    // 7. Return the final response
    Ok(models::ICloudResponse { metadata, photos })
}

#[cfg(test)]
mod tests {
    // Tests are in the separate test files
}