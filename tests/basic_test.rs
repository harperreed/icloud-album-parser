use icloud_album_rs::get_icloud_photos;

#[test]
fn test_get_icloud_photos_placeholder() {
    let result = get_icloud_photos("test_token").unwrap();
    assert!(result.contains("test_token"));
}