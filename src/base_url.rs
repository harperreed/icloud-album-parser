//! ABOUTME: This module handles the generation of base URLs for iCloud album API.
//! ABOUTME: It implements the token parsing logic to determine server partitions.

/// Converts a token character to a base62 value
fn char_to_base62(c: char) -> u32 {
    match c {
        '0'..='9' => c as u32 - '0' as u32,
        'A'..='Z' => c as u32 - 'A' as u32 + 10,
        'a'..='z' => c as u32 - 'a' as u32 + 36,
        _ => panic!("Invalid base62 character: {}", c),
    }
}

/// Calculate server partition based on the token's first character
fn calculate_partition(token: &str) -> u32 {
    if token.is_empty() {
        return 10; // Default partition if token is empty
    }
    
    // Get the first character of the token
    let first_char = token.chars().next().unwrap();
    
    // Convert to base62 value and use modulo to get a server partition between 1-40
    let base62_value = char_to_base62(first_char);
    1 + (base62_value % 40)
}

/// Generates the base URL for the iCloud API using the token
/// 
/// The URL is constructed in the format:
/// `https://pXX-sharedstreams.icloud.com/{token}/sharedstreams/`
/// where XX is the server partition determined by the first character of the token.
///
/// # Arguments
///
/// * `token` - The iCloud shared album token
///
/// # Returns
///
/// The generated base URL as a String
pub fn get_base_url(token: &str) -> String {
    let server_partition = calculate_partition(token);
    format!(
        "https://p{:02}-sharedstreams.icloud.com/{}/sharedstreams/",
        server_partition, token
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_to_base62() {
        // Test digits (0-9)
        assert_eq!(char_to_base62('0'), 0);
        assert_eq!(char_to_base62('9'), 9);
        
        // Test uppercase letters (A-Z)
        assert_eq!(char_to_base62('A'), 10);
        assert_eq!(char_to_base62('Z'), 35);
        
        // Test lowercase letters (a-z)
        assert_eq!(char_to_base62('a'), 36);
        assert_eq!(char_to_base62('z'), 61);
    }
    
    #[test]
    fn test_calculate_partition() {
        // Test with various first characters
        assert_eq!(calculate_partition("A0z5qAGN1JIFd3y"), 11); // A -> 10 -> 11
        assert_eq!(calculate_partition("B0z5qAGN1JIFd3y"), 12); // B -> 11 -> 12
        assert_eq!(calculate_partition("a0z5qAGN1JIFd3y"), 37); // a -> 36 -> 37
        assert_eq!(calculate_partition("z0z5qAGN1JIFd3y"), 22); // z -> 61 -> 22 (61 % 40 + 1)
        
        // Test with empty string should use default
        assert_eq!(calculate_partition(""), 10);
    }
    
    #[test]
    fn test_get_base_url() {
        // Complete URL test
        let token = "A0z5qAGN1JIFd3y";
        let expected = "https://p11-sharedstreams.icloud.com/A0z5qAGN1JIFd3y/sharedstreams/";
        assert_eq!(get_base_url(token), expected);
        
        // Different token
        let token = "B0z5qAGN1JIFd3y";
        let expected = "https://p12-sharedstreams.icloud.com/B0z5qAGN1JIFd3y/sharedstreams/";
        assert_eq!(get_base_url(token), expected);
    }
}