use icloud_album_rs::models::{ApiResponse, Derivative, ICloudResponse, Image, Metadata};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_derivative_deserialization() {
    let json_str = r#"
    {
        "checksum": "abc123",
        "fileSize": 12345,
        "width": 800,
        "height": 600,
        "url": "https://example.com/image.jpg"
    }
    "#;

    let derivative: Derivative = serde_json::from_str(json_str).unwrap();

    assert_eq!(derivative.checksum, "abc123");
    assert_eq!(derivative.file_size, Some(12345));
    assert_eq!(derivative.width, Some(800));
    assert_eq!(derivative.height, Some(600));
    assert_eq!(
        derivative.url,
        Some("https://example.com/image.jpg".to_string())
    );
}

#[test]
fn test_image_deserialization() {
    let json_str = r#"
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
        "caption": "Test image",
        "dateCreated": "2023-01-01",
        "batchDateCreated": "2023-01-01",
        "width": 1600,
        "height": 1200
    }
    "#;

    let image: Image = serde_json::from_str(json_str).unwrap();

    assert_eq!(image.photo_guid, "photo123");
    assert_eq!(image.derivatives.len(), 2);
    assert_eq!(image.derivatives.get("1").unwrap().checksum, "abc123");
    assert_eq!(image.derivatives.get("2").unwrap().checksum, "def456");
    assert_eq!(image.caption, Some("Test image".to_string()));
    assert_eq!(image.date_created, Some("2023-01-01".to_string()));
    assert_eq!(image.width, Some(1600));
    assert_eq!(image.height, Some(1200));
}

#[test]
fn test_metadata_deserialization() {
    let json_str = r#"
    {
        "streamName": "My Album",
        "userFirstName": "John",
        "userLastName": "Doe",
        "streamCtag": "ctag123",
        "itemsReturned": 10,
        "locations": {}
    }
    "#;

    let metadata: Metadata = serde_json::from_str(json_str).unwrap();

    assert_eq!(metadata.stream_name, "My Album");
    assert_eq!(metadata.user_first_name, "John");
    assert_eq!(metadata.user_last_name, "Doe");
    assert_eq!(metadata.stream_ctag, "ctag123");
    assert_eq!(metadata.items_returned, 10);
}

#[test]
fn test_api_response_deserialization() {
    let json_str = r#"
    {
        "photos": [
            {
                "photoGuid": "photo123",
                "derivatives": {
                    "1": {
                        "checksum": "abc123",
                        "fileSize": 12345,
                        "width": 800,
                        "height": 600
                    }
                },
                "caption": "Test image",
                "dateCreated": "2023-01-01",
                "width": 1600,
                "height": 1200
            }
        ],
        "photoGuids": ["photo123"],
        "streamName": "My Album",
        "userFirstName": "John",
        "userLastName": "Doe",
        "streamCtag": "ctag123",
        "itemsReturned": "10",
        "locations": {}
    }
    "#;

    let api_response: ApiResponse = serde_json::from_str(json_str).unwrap();

    assert_eq!(api_response.photos.len(), 1);
    assert_eq!(api_response.photo_guids.len(), 1);
    assert_eq!(api_response.photo_guids[0], "photo123");
    assert_eq!(api_response.stream_name, Some("My Album".to_string()));
    assert_eq!(api_response.user_first_name, Some("John".to_string()));
    assert_eq!(api_response.user_last_name, Some("Doe".to_string()));
    assert_eq!(api_response.stream_ctag, Some("ctag123".to_string()));
    assert_eq!(api_response.items_returned, Some("10".to_string()));
}

#[test]
fn test_icloud_response_construction() {
    // Create a minimal metadata instance
    let metadata = Metadata {
        stream_name: "My Album".to_string(),
        user_first_name: "John".to_string(),
        user_last_name: "Doe".to_string(),
        stream_ctag: "ctag123".to_string(),
        items_returned: 1,
        locations: json!({}),
    };

    // Create a minimal derivative
    let mut derivatives = HashMap::new();
    derivatives.insert(
        "1".to_string(),
        Derivative {
            checksum: "abc123".to_string(),
            file_size: Some(12345),
            width: Some(800),
            height: Some(600),
            url: Some("https://example.com/image.jpg".to_string()),
        },
    );

    // Create a minimal image
    let image = Image {
        photo_guid: "photo123".to_string(),
        derivatives,
        caption: Some("Test image".to_string()),
        date_created: Some("2023-01-01".to_string()),
        batch_date_created: Some("2023-01-01".to_string()),
        width: Some(1600),
        height: Some(1200),
    };

    // Create an ICloudResponse
    let icloud_response = ICloudResponse {
        metadata,
        photos: vec![image],
    };

    assert_eq!(icloud_response.metadata.stream_name, "My Album");
    assert_eq!(icloud_response.photos.len(), 1);
    assert_eq!(icloud_response.photos[0].photo_guid, "photo123");
}
