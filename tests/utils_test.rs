use icloud_album_rs::models::Derivative;
use icloud_album_rs::utils;
use std::collections::HashMap;

#[test]
fn test_extension_from_mime_type() {
    // Test known MIME types
    assert_eq!(utils::extension_from_mime_type("image/jpeg"), ".jpg");
    assert_eq!(utils::extension_from_mime_type("image/png"), ".png");
    assert_eq!(utils::extension_from_mime_type("image/heic"), ".heic");
    assert_eq!(utils::extension_from_mime_type("image/heif"), ".heif");
    assert_eq!(utils::extension_from_mime_type("video/mp4"), ".mp4");
    assert_eq!(utils::extension_from_mime_type("video/quicktime"), ".mov");
    assert_eq!(utils::extension_from_mime_type("image/gif"), ".gif");

    // Test unknown MIME type (should default to .jpg)
    assert_eq!(
        utils::extension_from_mime_type("application/octet-stream"),
        ".jpg"
    );
}

#[test]
fn test_detect_mime_type() {
    // JPEG test data (FF D8 FF)
    let jpeg_bytes = [
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
    ];
    assert_eq!(utils::detect_mime_type(&jpeg_bytes, None), "image/jpeg");

    // PNG test data (89 50 4E 47 0D 0A 1A 0A)
    let png_bytes = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
    ];
    assert_eq!(utils::detect_mime_type(&png_bytes, None), "image/png");

    // MP4 test data (ftyp at position 4)
    let mp4_bytes = [
        0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, 0x6D, 0x70, 0x34, 0x32,
    ];
    assert_eq!(utils::detect_mime_type(&mp4_bytes, None), "video/mp4");

    // MOV test data (ftyp qt at position 4-10)
    let mov_bytes = [
        0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, 0x71, 0x74, 0x20, 0x20,
    ];
    assert_eq!(utils::detect_mime_type(&mov_bytes, None), "video/quicktime");

    // GIF test data (GIF89a)
    let gif_bytes = [
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    assert_eq!(utils::detect_mime_type(&gif_bytes, None), "image/gif");

    // Invalid/short data (should default to image/jpeg)
    let invalid_bytes = [0x00, 0x01];
    assert_eq!(utils::detect_mime_type(&invalid_bytes, None), "image/jpeg");

    // Test with filename hint when content detection fails
    let unknown_bytes = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
    assert_ne!(
        utils::detect_mime_type(&unknown_bytes, Some("test.png")),
        "image/jpeg" // Should not be defaulting to JPEG with a PNG filename
    );
}

#[test]
fn test_select_best_derivative() {
    let mut derivatives = HashMap::new();

    // Add various derivatives
    let mut derivative1 = Derivative {
        checksum: "checksum1".to_string(),
        file_size: Some(10000),
        width: Some(800),
        height: Some(600),
        url: Some("https://example.com/image1.jpg".to_string()),
    };

    let mut derivative2 = Derivative {
        checksum: "checksum2".to_string(),
        file_size: Some(40000),
        width: Some(1600),
        height: Some(1200),
        url: Some("https://example.com/image2.jpg".to_string()),
    };

    let mut derivative3 = Derivative {
        checksum: "checksum3".to_string(),
        file_size: Some(100000),
        width: Some(3200),
        height: Some(2400),
        url: Some("https://example.com/image3.jpg".to_string()),
    };

    // Test 1: Basic resolution comparison
    derivatives.insert("1".to_string(), derivative1.clone());
    derivatives.insert("2".to_string(), derivative2.clone());

    let result = utils::select_best_derivative(&derivatives);
    assert!(result.is_some());
    let (key, _der, _url) = result.unwrap();
    assert_eq!(key, "2"); // Should pick the higher resolution

    // Test 2: With an "original" that has better resolution
    derivatives.clear();
    derivatives.insert("1".to_string(), derivative1.clone());
    derivatives.insert("2".to_string(), derivative2.clone());
    derivatives.insert("3".to_string(), derivative3.clone()); // iCloud often uses "3" for originals

    let result = utils::select_best_derivative(&derivatives);
    assert!(result.is_some());
    let (key, _der, _url) = result.unwrap();
    assert_eq!(key, "3"); // Should pick key "3" as it has highest resolution

    // Test 3: With a derivative missing dimensions
    derivatives.clear();
    derivative1.width = None;
    derivative1.height = None;
    derivatives.insert("1".to_string(), derivative1.clone());
    derivatives.insert("2".to_string(), derivative2.clone());

    let result = utils::select_best_derivative(&derivatives);
    assert!(result.is_some());
    let (key, _der, _url) = result.unwrap();
    assert_eq!(key, "2"); // Should skip derivative1 and pick derivative2

    // Test 4: With a derivative missing URL
    derivatives.clear();
    derivative2.url = None;
    derivatives.insert("1".to_string(), derivative1.clone());
    derivatives.insert("2".to_string(), derivative2.clone());

    let result = utils::select_best_derivative(&derivatives);
    assert!(result.is_some());
    let (key, _der, _url) = result.unwrap();
    assert_eq!(key, "1"); // Should skip derivative2 (no URL) and pick derivative1

    // Test 5: Empty derivatives map
    derivatives.clear();
    let result = utils::select_best_derivative(&derivatives);
    assert!(result.is_none());

    // Test 6: With "original" or "full" in the key name
    derivatives.clear();
    derivative2.width = Some(1600);
    derivative2.height = Some(1200);
    derivative2.url = Some("https://example.com/image2.jpg".to_string());
    derivative3.width = Some(1200);
    derivative3.height = Some(900);
    derivative3.url = Some("https://example.com/image3.jpg".to_string());

    derivatives.insert("small".to_string(), derivative1.clone());
    derivatives.insert("medium".to_string(), derivative3.clone());
    derivatives.insert("original".to_string(), derivative2.clone());

    let result = utils::select_best_derivative(&derivatives);
    assert!(result.is_some());
    let (key, _der, _url) = result.unwrap();
    assert_eq!(key, "original"); // Should prioritize the one with "original" in key
}
