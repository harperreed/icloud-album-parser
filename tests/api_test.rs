// We'll use a main function with #[tokio::main] to run tests
// The tokio::main attribute properly configures the runtime 
#[tokio::main]
async fn main() {
    // Run all tests and report success or failure
    let success = run_all_tests().await;
    assert!(success, "One or more tests failed");
}

async fn run_all_tests() -> bool {
    println!("Running API tests...");
    
    let api_response_success = test_get_api_response().await;
    println!("API response test: {}", if api_response_success { "PASSED" } else { "FAILED" });
    
    api_response_success
}

use icloud_album_rs::api::get_api_response;
use reqwest::Client;
use serde_json::json;

async fn test_get_api_response() -> bool {
    // Create a mock server
    let mut mock_server = mockito::Server::new();
    let mock_url = mock_server.url();
    
    // Create a sample response JSON with metadata and photos
    let sample_response = json!({
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
    });
    
    // Set up the mock response
    let _m = mock_server.mock("POST", "/webstream")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(sample_response.to_string())
        .create();
    
    // Test with a base URL that ends with the mock server URL plus a trailing slash
    let base_url = format!("{}/", mock_url);
    let client = Client::new();
    
    // Call the function and check the result
    match get_api_response(&client, &base_url).await {
        Ok((photos, metadata)) => {
            // Verify metadata
            let metadata_correct = 
                metadata.stream_name == "Test Album" &&
                metadata.user_first_name == "John" &&
                metadata.user_last_name == "Doe" &&
                metadata.stream_ctag == "12345" &&
                metadata.items_returned == 2;
            
            // Verify photos
            let photos_correct = 
                photos.len() == 2 &&
                photos[0].photo_guid == "photo123" &&
                photos[0].derivatives.len() == 2 &&
                photos[0].derivatives.get("1").map(|d| d.checksum.clone()) == Some("abc123".to_string()) &&
                photos[1].photo_guid == "photo456" &&
                photos[1].derivatives.len() == 1 &&
                photos[1].derivatives.get("1").map(|d| d.checksum.clone()) == Some("ghi789".to_string());
            
            metadata_correct && photos_correct
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            false
        }
    }
}