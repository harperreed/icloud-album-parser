use icloud_album_rs::base_url::get_base_url;

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
    
    // Test with empty token (should use default partition)
    let token = "";
    let expected = "https://p10-sharedstreams.icloud.com//sharedstreams/";
    assert_eq!(get_base_url(token).unwrap(), expected);
    
    // Test with invalid character (should use default partition)
    let token = "!invalid"; 
    let expected = "https://p10-sharedstreams.icloud.com/!invalid/sharedstreams/";
    assert_eq!(get_base_url(token).unwrap(), expected);
}