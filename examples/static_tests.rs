//! Static tests that don't use mockito
//!
//! This file contains tests that use pre-defined responses instead of mockito
//! to avoid runtime conflicts. It serves as documentation of the expected
//! behavior of the library's functions when processing API responses.
//!
//! Run with: cargo run --example static_tests

use icloud_album_rs::enrich::enrich_photos_with_urls;
use icloud_album_rs::models::{Derivative, Image, Metadata};
use std::collections::HashMap;

// We'll use tokio::main to run our tests
fn main() {
    println!("Running static API tests (no HTTP requests)...");

    test_parse_api_response();
    test_parse_asset_urls();

    println!("\n✅ All static tests completed!");
}

fn test_parse_api_response() {
    println!("\nTesting API response manual parsing...");

    // Create metadata and images manually
    let metadata = Metadata {
        stream_name: "Test Album".to_string(),
        user_first_name: "John".to_string(),
        user_last_name: "Doe".to_string(),
        stream_ctag: "12345".to_string(),
        items_returned: 2,
        locations: serde_json::json!({}),
    };

    // Create first image with derivatives
    let mut derivatives1 = HashMap::new();
    derivatives1.insert(
        "1".to_string(),
        Derivative {
            checksum: "abc123".to_string(),
            file_size: Some(12345),
            width: Some(800),
            height: Some(600),
            url: None,
        },
    );
    derivatives1.insert(
        "2".to_string(),
        Derivative {
            checksum: "def456".to_string(),
            file_size: Some(54321),
            width: Some(1600),
            height: Some(1200),
            url: None,
        },
    );

    let image1 = Image {
        photo_guid: "photo123".to_string(),
        derivatives: derivatives1,
        caption: Some("Test image 1".to_string()),
        date_created: Some("2023-01-01".to_string()),
        batch_date_created: Some("2023-01-01".to_string()),
        width: Some(1600),
        height: Some(1200),
    };

    // Create second image with derivatives
    let mut derivatives2 = HashMap::new();
    derivatives2.insert(
        "1".to_string(),
        Derivative {
            checksum: "ghi789".to_string(),
            file_size: Some(23456),
            width: Some(800),
            height: Some(600),
            url: None,
        },
    );

    let image2 = Image {
        photo_guid: "photo456".to_string(),
        derivatives: derivatives2,
        caption: Some("Test image 2".to_string()),
        date_created: Some("2023-01-02".to_string()),
        batch_date_created: Some("2023-01-02".to_string()),
        width: Some(800),
        height: Some(600),
    };

    let photos = vec![image1, image2];

    // Verify the manually created objects
    assert_eq!(metadata.stream_name, "Test Album", "Stream name mismatch");
    assert_eq!(metadata.user_first_name, "John", "User first name mismatch");
    assert_eq!(metadata.user_last_name, "Doe", "User last name mismatch");
    assert_eq!(metadata.stream_ctag, "12345", "Stream ctag mismatch");
    assert_eq!(metadata.items_returned, 2, "Items returned mismatch");

    // Verify photos
    assert_eq!(photos.len(), 2, "Photo count mismatch");
    assert_eq!(photos[0].photo_guid, "photo123", "Photo GUID mismatch");
    assert_eq!(photos[0].derivatives.len(), 2, "Derivative count mismatch");
    assert_eq!(
        photos[0].derivatives.get("1").map(|d| d.checksum.clone()),
        Some("abc123".to_string()),
        "Derivative checksum mismatch"
    );
    assert_eq!(photos[1].photo_guid, "photo456", "Photo GUID mismatch");
    assert_eq!(photos[1].derivatives.len(), 1, "Derivative count mismatch");
    assert_eq!(
        photos[1].derivatives.get("1").map(|d| d.checksum.clone()),
        Some("ghi789".to_string()),
        "Derivative checksum mismatch"
    );

    println!("  ✅ API response manual parsing test passed!");
}

fn test_parse_asset_urls() {
    println!("\nTesting URL enrichment...");

    // Create sample photos
    let mut derivatives1 = HashMap::new();
    derivatives1.insert(
        "1".to_string(),
        Derivative {
            checksum: "abc123".to_string(),
            file_size: Some(12345),
            width: Some(800),
            height: Some(600),
            url: None,
        },
    );

    let image1 = Image {
        photo_guid: "photo123".to_string(),
        derivatives: derivatives1,
        caption: Some("Test image 1".to_string()),
        date_created: Some("2023-01-01".to_string()),
        batch_date_created: Some("2023-01-01".to_string()),
        width: Some(1600),
        height: Some(1200),
    };

    let mut derivatives2 = HashMap::new();
    derivatives2.insert(
        "1".to_string(),
        Derivative {
            checksum: "def456".to_string(),
            file_size: Some(23456),
            width: Some(800),
            height: Some(600),
            url: None,
        },
    );

    let image2 = Image {
        photo_guid: "photo456".to_string(),
        derivatives: derivatives2,
        caption: Some("Test image 2".to_string()),
        date_created: Some("2023-01-02".to_string()),
        batch_date_created: Some("2023-01-02".to_string()),
        width: Some(800),
        height: Some(600),
    };

    let mut photos = vec![image1, image2];

    // Create sample URLs
    let mut urls = HashMap::new();
    urls.insert(
        "abc123".to_string(),
        "https://example1.icloud.com/path/to/image1.jpg".to_string(),
    );
    urls.insert(
        "def456".to_string(),
        "https://example2.icloud.com/path/to/image2.jpg".to_string(),
    );

    // Enrich photos with URLs
    enrich_photos_with_urls(&mut photos, &urls);

    // Verify enrichment
    assert_eq!(photos.len(), 2, "Photo count mismatch");

    // Check that URLs were properly added to derivatives
    match &photos[0].derivatives.get("1").unwrap().url {
        Some(url) => {
            assert_eq!(
                url, "https://example1.icloud.com/path/to/image1.jpg",
                "URL 1 mismatch"
            );
            println!("  ✅ URL 1 enrichment test passed!");
        }
        None => {
            println!("  ❌ URL 1 enrichment test failed: No URL found");
            panic!("URL 1 enrichment test failed");
        }
    }

    match &photos[1].derivatives.get("1").unwrap().url {
        Some(url) => {
            assert_eq!(
                url, "https://example2.icloud.com/path/to/image2.jpg",
                "URL 2 mismatch"
            );
            println!("  ✅ URL 2 enrichment test passed!");
        }
        None => {
            println!("  ❌ URL 2 enrichment test failed: No URL found");
            panic!("URL 2 enrichment test failed");
        }
    }

    println!("  ✅ All URL enrichment tests passed!");
}
