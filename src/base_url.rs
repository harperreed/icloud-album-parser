//! URL generation for iCloud album API endpoints.
//!
//! This module handles base URL construction and token parsing to determine
//! the correct server partition for API requests.

/// Error type for base URL generation
#[derive(Debug, thiserror::Error)]
pub enum BaseUrlError {
    #[error("Invalid base62 character: {0}")]
    InvalidBase62Char(char),
    #[error("Empty token provided")]
    EmptyToken,
}

/// Converts a token character to a base62 value
fn char_to_base62(c: char) -> Result<u32, BaseUrlError> {
    match c {
        '0'..='9' => Ok(c as u32 - '0' as u32),
        'A'..='Z' => Ok(c as u32 - 'A' as u32 + 10),
        'a'..='z' => Ok(c as u32 - 'a' as u32 + 36),
        _ => Err(BaseUrlError::InvalidBase62Char(c)),
    }
}

/// Calculate server partition based on the token's first character
fn calculate_partition(token: &str) -> Result<u32, BaseUrlError> {
    if token.is_empty() {
        return Err(BaseUrlError::EmptyToken);
    }
    
    // Get the first character of the token
    let first_char = token.chars().next().ok_or(BaseUrlError::EmptyToken)?;
    
    // Convert to base62 value and use modulo to get a server partition between 1-40
    let base62_value = char_to_base62(first_char)?;
    Ok(1 + (base62_value % 40))
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
/// The generated base URL as a Result containing either the URL string or an error
pub fn get_base_url(token: &str) -> Result<String, BaseUrlError> {
    let server_partition = calculate_partition(token)?;
    Ok(format!(
        "https://p{:02}-sharedstreams.icloud.com/{}/sharedstreams/",
        server_partition, token
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_to_base62() {
        // Test digits (0-9)
        assert_eq!(char_to_base62('0').unwrap(), 0);
        assert_eq!(char_to_base62('9').unwrap(), 9);
        
        // Test uppercase letters (A-Z)
        assert_eq!(char_to_base62('A').unwrap(), 10);
        assert_eq!(char_to_base62('Z').unwrap(), 35);
        
        // Test lowercase letters (a-z)
        assert_eq!(char_to_base62('a').unwrap(), 36);
        assert_eq!(char_to_base62('z').unwrap(), 61);
        
        // Test invalid character
        assert!(char_to_base62('!').is_err());
        match char_to_base62('!') {
            Err(BaseUrlError::InvalidBase62Char(c)) => assert_eq!(c, '!'),
            _ => panic!("Expected InvalidBase62Char error"),
        }
    }
    
    #[test]
    fn test_calculate_partition() {
        // Test with various first characters
        assert_eq!(calculate_partition("A0z5qAGN1JIFd3y").unwrap(), 11); // A -> 10 -> 11
        assert_eq!(calculate_partition("B0z5qAGN1JIFd3y").unwrap(), 12); // B -> 11 -> 12
        assert_eq!(calculate_partition("a0z5qAGN1JIFd3y").unwrap(), 37); // a -> 36 -> 37
        assert_eq!(calculate_partition("z0z5qAGN1JIFd3y").unwrap(), 22); // z -> 61 -> 22 (61 % 40 + 1)
        
        // Test with empty string should return error
        assert!(calculate_partition("").is_err());
        match calculate_partition("") {
            Err(BaseUrlError::EmptyToken) => (), // Success
            _ => panic!("Expected EmptyToken error"),
        }
        
        // Test with invalid character
        assert!(calculate_partition("!abc").is_err());
        match calculate_partition("!abc") {
            Err(BaseUrlError::InvalidBase62Char(c)) => assert_eq!(c, '!'),
            _ => panic!("Expected InvalidBase62Char error"),
        }
    }
    
    #[test]
    fn test_get_base_url() {
        // Complete URL test
        let token = "A0z5qAGN1JIFd3y";
        let expected = "https://p11-sharedstreams.icloud.com/A0z5qAGN1JIFd3y/sharedstreams/";
        assert_eq!(get_base_url(token).unwrap(), expected);
        
        // Different token
        let token = "B0z5qAGN1JIFd3y";
        let expected = "https://p12-sharedstreams.icloud.com/B0z5qAGN1JIFd3y/sharedstreams/";
        assert_eq!(get_base_url(token).unwrap(), expected);
        
        // Test with empty string should now return an error
        let token = "";
        assert!(matches!(get_base_url(token), Err(BaseUrlError::EmptyToken)));
        
        // Test with invalid character should now return an error
        let token = "!invalid";
        assert!(matches!(get_base_url(token), Err(BaseUrlError::InvalidBase62Char('!'))));
    }
}