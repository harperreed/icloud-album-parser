//! ABOUTME: Utility functions for file operations and media handling
//! ABOUTME: Contains functions for MIME type detection, file extension mapping, and other utilities

use crate::models::Derivative;
use log::{debug, warn};
use mime_guess::from_path;
use std::collections::HashMap;

/// Returns the appropriate file extension based on MIME type
///
/// # Arguments
///
/// * `mime_type` - The MIME type string
///
/// # Returns
///
/// A string containing the appropriate file extension with leading dot
pub fn extension_from_mime_type(mime_type: &str) -> String {
    match mime_type {
        "image/jpeg" => ".jpg".to_string(),
        "image/png" => ".png".to_string(),
        "image/heic" => ".heic".to_string(),
        "image/heif" => ".heif".to_string(),
        "video/mp4" => ".mp4".to_string(),
        "video/quicktime" => ".mov".to_string(),
        "image/gif" => ".gif".to_string(),
        _ => {
            warn!("Unknown MIME type: {}, defaulting to .jpg", mime_type);
            ".jpg".to_string()
        }
    }
}

/// Detects MIME type from content bytes
///
/// # Arguments
///
/// * `bytes` - The content bytes to analyze
/// * `filename` - Optional filename to provide additional context
///
/// # Returns
///
/// A string containing the detected MIME type
pub fn detect_mime_type(bytes: &[u8], filename: Option<&str>) -> String {
    // Common image and video file signatures
    if bytes.len() >= 12 {
        // JPEG: Starts with FF D8 FF
        if bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
            return "image/jpeg".to_string();
        }

        // PNG: Starts with 89 50 4E 47 0D 0A 1A 0A
        if bytes[0] == 0x89
            && bytes[1] == 0x50
            && bytes[2] == 0x4E
            && bytes[3] == 0x47
            && bytes[4] == 0x0D
            && bytes[5] == 0x0A
            && bytes[6] == 0x1A
            && bytes[7] == 0x0A
        {
            return "image/png".to_string();
        }

        // Check for MOV first (more specific) - ftyp at bytes 4-8 with qt in next position
        if bytes.len() > 11
            && bytes[4] == 0x66
            && bytes[5] == 0x74
            && bytes[6] == 0x79
            && bytes[7] == 0x70
            && bytes[8] == 0x71
            && bytes[9] == 0x74
        {
            return "video/quicktime".to_string();
        }

        // MP4: ftyp at bytes 4-8 (more general)
        if bytes.len() > 11
            && bytes[4] == 0x66
            && bytes[5] == 0x74
            && bytes[6] == 0x79
            && bytes[7] == 0x70
        {
            return "video/mp4".to_string();
        }

        // GIF: Starts with GIF87a or GIF89a
        if bytes.len() >= 6
            && bytes[0] == 0x47
            && bytes[1] == 0x49
            && bytes[2] == 0x46
            && bytes[3] == 0x38
            && (bytes[4] == 0x37 || bytes[4] == 0x39)
            && bytes[5] == 0x61
        {
            return "image/gif".to_string();
        }

        // HEIC/HEIF detection
        if bytes.len() > 12
            && bytes[4] == 0x66
            && bytes[5] == 0x74
            && bytes[6] == 0x79
            && bytes[7] == 0x70
            && bytes[8] == 0x68
            && bytes[9] == 0x65
            && bytes[10] == 0x69
            && (bytes[11] == 0x63 || bytes[11] == 0x66)
        {
            // Determine if it's HEIC or HEIF based on the last identifier byte
            if bytes[11] == 0x63 {
                return "image/heic".to_string();
            } else {
                return "image/heif".to_string();
            }
        }
    }

    // If we couldn't detect from bytes, try to use the filename
    if let Some(name) = filename {
        let mime = from_path(name).first_or_octet_stream();
        return mime.to_string();
    }

    // Default to JPEG if we couldn't detect
    debug!("Could not detect MIME type, defaulting to image/jpeg");
    "image/jpeg".to_string()
}

/// Returns the appropriate file extension for the given content
///
/// # Arguments
///
/// * `bytes` - The content bytes to analyze
/// * `filename` - Optional filename to provide additional context
///
/// # Returns
///
/// A string containing the appropriate file extension with leading dot
pub fn get_extension_for_content(bytes: &[u8], filename: Option<&str>) -> String {
    let mime_type = detect_mime_type(bytes, filename);
    extension_from_mime_type(&mime_type)
}

/// Selects the best derivative based on resolution and other criteria
///
/// This function implements a smarter algorithm for selecting the best derivative:
/// 1. Prioritize derivatives with both width and height defined
/// 2. Prefer originals when available
/// 3. Among derivatives with dimensions, choose the one with the highest resolution
///
/// # Arguments
///
/// * `derivatives` - HashMap of derivative key to Derivative
///
/// # Returns
///
/// An Option containing the derivative key, Derivative, and URL if found
pub fn select_best_derivative(
    derivatives: &HashMap<String, Derivative>,
) -> Option<(String, &Derivative, String)> {
    // Guard against empty derivatives
    if derivatives.is_empty() {
        return None;
    }

    // Check if we have an original with dimensions
    let mut best_derivative = None;
    let mut max_resolution = 0;
    let mut has_original = false;

    // First pass: check for original or highest resolution with dimensions
    for (key, derivative) in derivatives {
        // Skip derivatives without URLs
        if derivative.url.is_none() {
            continue;
        }

        let url = derivative.url.as_ref().unwrap();

        // Check if this is likely an original (by key name pattern)
        let is_original = key.to_lowercase().contains("original") || 
                          key.to_lowercase().contains("full") || 
                          key == "3" || // iCloud often uses "3" as original
                          key == "4"; // Sometimes "4" is the highest quality

        if is_original {
            has_original = true;

            // If original has dimensions, it's a prime candidate
            if let (Some(width), Some(height)) = (derivative.width, derivative.height) {
                let resolution = width as u64 * height as u64;

                // Prioritize originals with dimensions
                if resolution > max_resolution {
                    max_resolution = resolution;
                    best_derivative = Some((key.clone(), derivative, url.clone()));
                }
            }
            // Even without dimensions, consider it if we don't have better options
            else if best_derivative.is_none() {
                best_derivative = Some((key.clone(), derivative, url.clone()));
            }
        }
        // For non-originals with dimensions, track them as potential backups
        else if let (Some(width), Some(height)) = (derivative.width, derivative.height) {
            let resolution = width as u64 * height as u64;

            // If we don't have a good original yet, this might be our best option
            if resolution > max_resolution && !has_original {
                max_resolution = resolution;
                best_derivative = Some((key.clone(), derivative, url.clone()));
            }
        }
    }

    // If we didn't find anything with dimensions but have derivatives with URLs,
    // just pick the first one with a URL
    if best_derivative.is_none() {
        for (key, derivative) in derivatives {
            if let Some(url) = &derivative.url {
                return Some((key.clone(), derivative, url.clone()));
            }
        }
    }

    best_derivative
}
