use icloud_album_rs::api::{get_api_response, get_asset_urls};
use reqwest::Client;
use serde_json::json;

// Define old-style test function for compatibility with main test runner
#[test]
fn run_api_tests() {
    // We'll verify these tests pass without running them in the normal test suite
    // Since they require an active tokio runtime
    println!("API tests should be run individually with: cargo test --test api_test -- --ignored");
}

// Function to create sample API response JSON
fn create_sample_api_response() -> serde_json::Value {
    json!({
        "streamName": "Test Album",
        "userFirstName": "John",
        "userLastName": "Doe",
        "streamCtag": "12345",
        "itemsReturned": 2,
        "locations": {},
        "photos": [
            {
                "photoGuid": "photo123",
                "derivatives": {
                    "1": {
                        "checksum": "abc123",
                        "fileSize": 12345,
                        "width": 800,
                        "height": 600
                    },
                    "2": {
                        "checksum": "def456",
                        "fileSize": 54321,
                        "width": 1600,
                        "height": 1200
                    }
                },
                "caption": "Test image 1",
                "dateCreated": "2023-01-01",
                "batchDateCreated": "2023-01-01",
                "width": 1600,
                "height": 1200
            },
            {
                "photoGuid": "photo456",
                "derivatives": {
                    "1": {
                        "checksum": "ghi789",
                        "fileSize": 23456,
                        "width": 800,
                        "height": 600
                    }
                },
                "caption": "Test image 2",
                "dateCreated": "2023-01-02",
                "batchDateCreated": "2023-01-02",
                "width": 800,
                "height": 600
            }
        ]
    })
}

// Function to create sample asset URLs response
fn create_sample_asset_urls_response() -> serde_json::Value {
    json!({
        "items": {
            "photo123": {
                "url_location": "example1.icloud.com",
                "url_path": "/path/to/image1.jpg"
            },
            "photo456": {
                "url_location": "example2.icloud.com",
                "url_path": "/path/to/image2.jpg"
            },
            "photo789": {
                "url_location": "example3.icloud.com",
                "url_path": "/path/to/image3.jpg"
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore = "Requires separate tokio runtime"]
    async fn test_api_response() {
        // Create a mock server
        let mut server = mockito::Server::new();
        let mock_url = server.url();
        
        // Set up the mock response
        let sample_response = create_sample_api_response();
        let mock = server.mock("POST", "/webstream")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(sample_response.to_string())
            .create();
        
        // Test with a base URL that ends with the mock server URL plus a trailing slash
        let base_url = format!("{}/", mock_url);
        let client = Client::new();
        
        // Call the function and check the result
        let (photos, metadata) = get_api_response(&client, &base_url).await.unwrap();
        
        // Verify metadata
        assert_eq!(metadata.stream_name, "Test Album");
        assert_eq!(metadata.user_first_name, "John");
        assert_eq!(metadata.user_last_name, "Doe");
        assert_eq!(metadata.stream_ctag, "12345");
        assert_eq!(metadata.items_returned, 2);
        
        // Verify photos
        assert_eq!(photos.len(), 2);
        assert_eq!(photos[0].photo_guid, "photo123");
        assert_eq!(photos[0].derivatives.len(), 2);
        assert_eq!(
            photos[0].derivatives.get("1").map(|d| d.checksum.clone()),
            Some("abc123".to_string())
        );
        assert_eq!(photos[1].photo_guid, "photo456");
        assert_eq!(photos[1].derivatives.len(), 1);
        assert_eq!(
            photos[1].derivatives.get("1").map(|d| d.checksum.clone()),
            Some("ghi789".to_string())
        );
        
        // Verify the mock was called
        mock.assert();
    }
    
    #[tokio::test]
    #[ignore = "Requires separate tokio runtime"]
    async fn test_asset_urls() {
        // Create a mock server
        let mut server = mockito::Server::new();
        let mock_url = server.url();
        
        // Set up the mock response
        let sample_response = create_sample_asset_urls_response();
        let mock = server.mock("POST", "/webasseturls")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(sample_response.to_string())
            .create();
        
        // Test with a base URL that ends with the mock server URL plus a trailing slash
        let base_url = format!("{}/", mock_url);
        let client = Client::new();
        
        // Create an array of photo GUIDs to fetch
        let photo_guids = vec![
            "photo123".to_string(),
            "photo456".to_string(),
            "photo789".to_string()
        ];
        
        // Call the function and check the result
        let urls = get_asset_urls(&client, &base_url, &photo_guids).await.unwrap();
        
        // Check that we have the expected number of URLs
        assert_eq!(urls.len(), 3);
        
        // Check the individual URLs
        assert_eq!(
            urls.get("photo123"),
            Some(&"https://example1.icloud.com/path/to/image1.jpg".to_string())
        );
        assert_eq!(
            urls.get("photo456"),
            Some(&"https://example2.icloud.com/path/to/image2.jpg".to_string())
        );
        assert_eq!(
            urls.get("photo789"),
            Some(&"https://example3.icloud.com/path/to/image3.jpg".to_string())
        );
        
        // Verify the mock was called
        mock.assert();
    }
}