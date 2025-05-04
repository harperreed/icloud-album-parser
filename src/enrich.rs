//! Photo data enrichment functions.
//!
//! This module provides utilities to enrich photo data with additional information,
//! particularly combining photo metadata with their corresponding asset URLs
//! after they've been fetched from separate API endpoints.

use crate::models::Image;
use std::collections::HashMap;

/// Enriches photos by adding URLs to their derivatives
///
/// This function takes a mutable slice of Images and a HashMap of checksums to URLs,
/// and populates the URL field of each derivative in each Image if its checksum
/// matches one in the HashMap.
///
/// # Arguments
///
/// * `photos` - A mutable slice of Images to be enriched
/// * `all_urls` - A HashMap mapping from checksums to URLs
pub fn enrich_photos_with_urls(photos: &mut [Image], all_urls: &HashMap<String, String>) {
    // For each photo in the slice
    for photo in photos.iter_mut() {
        // For each derivative in the photo
        for derivative in photo.derivatives.values_mut() {
            // If the derivative's checksum is in the URL map
            if let Some(url) = all_urls.get(&derivative.checksum) {
                // Set the derivative's URL to the one from the map
                derivative.url = Some(url.to_string());
            }
        }
    }
}
