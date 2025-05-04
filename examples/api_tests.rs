use icloud_album_rs::api::{get_api_response, get_asset_urls};
use reqwest::Client;
use serde_json::json;

// This example file demonstrates how to run API tests with mockito
// Run with: cargo run --example api_tests

// We'll use tokio::main but configure it to use a multi-threaded runtime
// to avoid conflicts with mockito, which expects its own runtime
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    println!("Running API tests...");

    let api_test_passed = test_get_api_response().await;
    if api_test_passed {
        println!("✅ API response test passed!");
    } else {
        println!("❌ API response test failed!");
    }

    let assets_test_passed = test_get_asset_urls().await;
    if assets_test_passed {
        println!("✅ Asset URLs test passed!");
    } else {
        println!("❌ Asset URLs test failed!");
    }

    if api_test_passed && assets_test_passed {
        println!("✅ All API tests passed!");
    } else {
        println!("❌ Some tests failed!");
        std::process::exit(1);
    }
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

async fn test_get_api_response() -> bool {
    println!("Testing get_api_response...");

    // Create a mock server
    let mut server = mockito::Server::new();
    let mock_url = server.url();

    // Set up the mock response
    let sample_response = create_sample_api_response();
    let mock = server
        .mock("POST", "/webstream")
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
            let metadata_correct = metadata.stream_name == "Test Album"
                && metadata.user_first_name == "John"
                && metadata.user_last_name == "Doe"
                && metadata.stream_ctag == "12345"
                && metadata.items_returned == 2;

            // Verify photos
            let photos_correct = photos.len() == 2
                && photos[0].photo_guid == "photo123"
                && photos[0].derivatives.len() == 2
                && photos[0].derivatives.get("1").map(|d| d.checksum.clone())
                    == Some("abc123".to_string())
                && photos[1].photo_guid == "photo456"
                && photos[1].derivatives.len() == 1
                && photos[1].derivatives.get("1").map(|d| d.checksum.clone())
                    == Some("ghi789".to_string());

            // Verify the mock was called
            let mock_called = mock.matched();

            if !metadata_correct {
                println!("  ❌ Metadata verification failed");
            }
            if !photos_correct {
                println!("  ❌ Photos verification failed");
            }
            if !mock_called {
                println!("  ❌ Mock was not called");
            }

            metadata_correct && photos_correct && mock_called
        }
        Err(e) => {
            println!("  ❌ API request failed: {:?}", e);
            false
        }
    }
}

async fn test_get_asset_urls() -> bool {
    println!("Testing get_asset_urls...");

    // Create a mock server
    let mut server = mockito::Server::new();
    let mock_url = server.url();

    // Set up the mock response
    let sample_response = create_sample_asset_urls_response();
    let mock = server
        .mock("POST", "/webasseturls")
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
        "photo789".to_string(),
    ];

    // Call the function and check the result
    match get_asset_urls(&client, &base_url, &photo_guids).await {
        Ok(urls) => {
            // Check that we have the expected number of URLs
            let urls_count_correct = urls.len() == 3;

            // Check the individual URLs
            let url1_correct = urls.get("photo123")
                == Some(&"https://example1.icloud.com/path/to/image1.jpg".to_string());
            let url2_correct = urls.get("photo456")
                == Some(&"https://example2.icloud.com/path/to/image2.jpg".to_string());
            let url3_correct = urls.get("photo789")
                == Some(&"https://example3.icloud.com/path/to/image3.jpg".to_string());

            // Verify the mock was called
            let mock_called = mock.matched();

            if !urls_count_correct {
                println!("  ❌ Expected 3 URLs, got {}", urls.len());
            }
            if !url1_correct || !url2_correct || !url3_correct {
                println!("  ❌ URL verification failed");
            }
            if !mock_called {
                println!("  ❌ Mock was not called");
            }

            urls_count_correct && url1_correct && url2_correct && url3_correct && mock_called
        }
        Err(e) => {
            println!("  ❌ Error in asset URLs test: {}", e);
            false
        }
    }
}
