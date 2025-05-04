use icloud_album_rs::base_url::{get_base_url, BaseUrlError};

#[test]
fn test_get_base_url_with_different_tokens() {
    // Test with first token
    let token = "A0z5qAGN1JIFd3y";
    let expected = "https://p11-sharedstreams.icloud.com/A0z5qAGN1JIFd3y/sharedstreams/";
    assert_eq!(get_base_url(token).unwrap(), expected);
    
    // Test with second token
    let token = "B0z5qAGN1JIFd3y";
    let expected = "https://p12-sharedstreams.icloud.com/B0z5qAGN1JIFd3y/sharedstreams/";
    assert_eq!(get_base_url(token).unwrap(), expected);
    
    // Test with lowercase starting character
    let token = "a0z5qAGN1JIFd3y";
    let expected = "https://p37-sharedstreams.icloud.com/a0z5qAGN1JIFd3y/sharedstreams/";
    assert_eq!(get_base_url(token).unwrap(), expected);
    
    // Test with numeric starting character
    let token = "1234567890";
    let expected = "https://p02-sharedstreams.icloud.com/1234567890/sharedstreams/";
    assert_eq!(get_base_url(token).unwrap(), expected); // '1' -> 1 -> 2 (1 % 40 + 1)
}

#[test]
fn test_get_base_url_with_invalid_input() {
    // Test with empty token (should now return error)
    let token = "";
    match get_base_url(token) {
        Err(BaseUrlError::EmptyToken) => (),
        other => panic!("Expected EmptyToken error, got {:?}", other),
    }
    
    // Test with invalid character (should now return error)
    let token = "!invalid"; 
    match get_base_url(token) {
        Err(BaseUrlError::InvalidBase62Char(c)) => assert_eq!(c, '!'),
        other => panic!("Expected InvalidBase62Char error, got {:?}", other),
    }
}