//! A Rust library for interacting with iCloud shared albums.
//!
//! This library provides functionality to fetch, parse, and download photos from
//! iCloud shared albums using their public share token. It handles the various
//! API endpoints, authentication, redirects, and data parsing needed to work with
//! Apple's iCloud shared album service.
//!
//! # Logging
//!
//! This library uses the [`log`] crate for logging. You can enable logging by
//! initializing a logger in your application, such as [`env_logger`]. Set the
//! RUST_LOG environment variable to control log levels (e.g., `RUST_LOG=info`).
//!
//! ```
//! // Initialize the logger in your application
//! env_logger::init();
//! ```
//!
//! The library logs warnings for non-critical issues, such as type inconsistencies
//! in API responses, and errors for more serious problems.

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

/// Module containing utility functions for file handling
pub mod utils;

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
pub async fn get_icloud_photos(
    token: &str,
) -> Result<models::ICloudResponse, Box<dyn std::error::Error>> {
    // Create a reqwest client
    let client = reqwest::Client::new();

    // 1. Compute the base URL from the token
    let base_url = base_url::get_base_url(token)?;

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

/// Downloads a single photo or video from a shared album
///
/// This function:
/// 1. Selects the best derivative using the improved algorithm
/// 2. Downloads the content and detects the MIME type
/// 3. Determines the appropriate file extension
/// 4. Creates a file with the correct extension and saves the content
///
/// # Arguments
///
/// * `photo` - The photo to download
/// * `index` - Optional index for numbering purposes (useful in loops)
/// * `output_dir` - Directory where the file should be saved
/// * `custom_filename` - Optional custom filename to use (without extension)
///
/// # Returns
///
/// A Result containing the filepath where the content was saved
pub async fn download_photo(
    photo: &models::Image,
    index: Option<usize>,
    output_dir: &str,
    custom_filename: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    // Create a client for downloading
    let client = reqwest::Client::new();

    // Select the best derivative
    let best_derivative = utils::select_best_derivative(&photo.derivatives)
        .ok_or_else(|| "No suitable derivative found for download".to_string())?;

    // Extract components - we only need the URL
    let (_key, _derivative, url) = best_derivative;

    // Download the file content
    let response = client.get(&url).send().await?;
    let content = response.bytes().await?;

    // Get content type and appropriate extension
    let extension = utils::get_extension_for_content(&content, None);

    // Create the directory if it doesn't exist (using async tokio fs)
    if !tokio::fs::metadata(output_dir).await.is_ok() {
        tokio::fs::create_dir_all(output_dir).await?;
    }

    // Determine base filename
    let base_filename = if let Some(custom_name) = custom_filename {
        // Always include the photo_guid for uniqueness even with custom filenames
        format!("{}_{}", photo.photo_guid, custom_name)
    } else if let Some(caption) = &photo.caption {
        // Sanitize the caption for use as a filename - simplified version
        let sanitized = caption
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                _ => c,
            })
            .collect::<String>();

        if let Some(idx) = index {
            format!("{}_{}_{}", idx + 1, photo.photo_guid, sanitized)
        } else {
            format!("{}_{}", photo.photo_guid, sanitized)
        }
    } else if let Some(idx) = index {
        format!("{}_{}", idx + 1, photo.photo_guid)
    } else {
        photo.photo_guid.clone()
    };

    // Combine with extension
    let filename = format!("{}{}", base_filename, extension);
    let filepath = format!("{}/{}", output_dir, filename);

    // Write the file using async I/O
    let mut file = tokio::fs::File::create(&filepath).await?;
    tokio::io::copy(&mut content.as_ref(), &mut file).await?;

    Ok(filepath)
}

#[cfg(test)]
mod tests {
    // Tests are in the separate test files
}
