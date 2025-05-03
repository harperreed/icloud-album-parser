//! ABOUTME: This library provides functionality to interact with iCloud shared albums.
//! ABOUTME: It allows fetching metadata and photos from an album using a shared token.

/// Module containing data model structures
pub mod models;

/// Module handling the base URL generation for API calls
pub mod base_url;

/// Module handling redirects from the iCloud API
pub mod redirect;

/// This placeholder function will be replaced later with the actual implementation.
pub fn get_icloud_photos(token: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(format!("Placeholder for fetching photos with token: {}", token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = get_icloud_photos("test_token").unwrap();
        assert!(result.contains("test_token"));
    }
}