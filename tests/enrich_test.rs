use icloud_album_rs::enrich::enrich_photos_with_urls;
use icloud_album_rs::models::{Derivative, Image};
use std::collections::HashMap;

#[test]
fn test_enrich_photos_with_urls() {
    // Create a HashMap of checksums to URLs
    let mut all_urls = HashMap::new();
    all_urls.insert(
        "checksum1".to_string(),
        "https://example.com/image1.jpg".to_string(),
    );
    all_urls.insert(
        "checksum2".to_string(),
        "https://example.com/image2.jpg".to_string(),
    );
    all_urls.insert(
        "checksum3".to_string(),
        "https://example.com/image3.jpg".to_string(),
    );

    // Create derivatives with checksums
    let derivative1 = Derivative {
        checksum: "checksum1".to_string(),
        file_size: Some(12345),
        width: Some(800),
        height: Some(600),
        url: None,
    };

    let derivative2 = Derivative {
        checksum: "checksum2".to_string(),
        file_size: Some(23456),
        width: Some(1600),
        height: Some(1200),
        url: None,
    };

    let derivative3 = Derivative {
        checksum: "checksum3".to_string(),
        file_size: Some(34567),
        width: Some(2400),
        height: Some(1800),
        url: None,
    };

    let derivative4 = Derivative {
        checksum: "checksum4".to_string(), // This one doesn't have a URL in the map
        file_size: Some(45678),
        width: Some(3200),
        height: Some(2400),
        url: None,
    };

    // Create photos with derivatives
    let mut derivatives1 = HashMap::new();
    derivatives1.insert("1".to_string(), derivative1);
    derivatives1.insert("2".to_string(), derivative2);

    let mut derivatives2 = HashMap::new();
    derivatives2.insert("1".to_string(), derivative3);
    derivatives2.insert("2".to_string(), derivative4);

    let photo1 = Image {
        photo_guid: "photo1".to_string(),
        derivatives: derivatives1,
        caption: Some("Photo 1".to_string()),
        date_created: Some("2023-01-01".to_string()),
        batch_date_created: Some("2023-01-01".to_string()),
        width: Some(1600),
        height: Some(1200),
    };

    let photo2 = Image {
        photo_guid: "photo2".to_string(),
        derivatives: derivatives2,
        caption: Some("Photo 2".to_string()),
        date_created: Some("2023-01-02".to_string()),
        batch_date_created: Some("2023-01-02".to_string()),
        width: Some(2400),
        height: Some(1800),
    };

    // Create a mutable slice of photos
    let mut photos = vec![photo1, photo2];

    // Enrich the photos with URLs
    enrich_photos_with_urls(&mut photos, &all_urls);

    // Check that the URLs were correctly assigned
    assert_eq!(
        photos[0].derivatives.get("1").unwrap().url,
        Some("https://example.com/image1.jpg".to_string())
    );

    assert_eq!(
        photos[0].derivatives.get("2").unwrap().url,
        Some("https://example.com/image2.jpg".to_string())
    );

    assert_eq!(
        photos[1].derivatives.get("1").unwrap().url,
        Some("https://example.com/image3.jpg".to_string())
    );

    // This derivative shouldn't have a URL since its checksum wasn't in the map
    assert_eq!(photos[1].derivatives.get("2").unwrap().url, None);
}
