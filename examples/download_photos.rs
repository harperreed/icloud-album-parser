//! Example of downloading photos from an iCloud shared album
//!
//! Run with:
//! ```
//! cargo run --example download_photos -- "your_shared_album_token" "./download_dir"
//! ```

use icloud_album_rs::{download_photo, get_icloud_photos};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;

/// Sanitizes a filename to ensure it's valid across different operating systems
///
/// Replaces invalid characters with underscores and trims the filename if it's too long.
/// Common invalid characters are replaced, including:
/// - Control characters
/// - Characters that are illegal on various file systems (Windows, macOS, Linux)
/// - Characters that have special meaning in shell commands
fn sanitize_filename(input: &str) -> String {
    // Define invalid characters for filenames across different OS
    let mut invalid_chars = HashSet::new();

    // Control characters (0-31) and special characters
    for c in (0..32).map(|i| char::from_u32(i).unwrap_or(' ')) {
        invalid_chars.insert(c);
    }

    // Characters illegal in Windows filenames
    for c in &['<', '>', ':', '"', '/', '\\', '|', '?', '*'] {
        invalid_chars.insert(*c);
    }

    // Other potentially problematic characters
    for c in &[
        '!', '@', '#', '$', '%', '^', '&', '\'', ';', '=', '+', ',', '`', '~',
    ] {
        invalid_chars.insert(*c);
    }

    // Replace all invalid characters with underscores
    let sanitized = input
        .chars()
        .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
        .collect::<String>();

    // Remove leading/trailing dots and whitespace
    let sanitized = sanitized.trim().trim_matches('.').to_string();

    // Limit the filename length to a reasonable size (255 is often the max)
    // Leave room for the extension and potential path components
    if sanitized.len() > 200 {
        format!("{}_truncated", &sanitized[0..195])
    } else {
        sanitized
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the token and download directory from the command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: cargo run --example download_photos -- \"your_shared_album_token\" \"./download_dir\"");
        std::process::exit(1);
    }

    let token = &args[1];
    let download_dir = &args[2];

    // Create the download directory if it doesn't exist
    if !Path::new(download_dir).exists() {
        fs::create_dir_all(download_dir)?;
    }

    println!("Fetching shared album with token: {}", token);
    println!("Downloading photos to: {}", download_dir);

    // Fetch photos and metadata
    let response = get_icloud_photos(token).await?;

    println!("\nAlbum: {}", response.metadata.stream_name);
    println!(
        "Owner: {} {}",
        response.metadata.user_first_name, response.metadata.user_last_name
    );
    println!("Photos to download: {}", response.photos.len());

    // We don't need to create a client here anymore since download_photo creates its own

    // Download each photo
    for (i, photo) in response.photos.iter().enumerate() {
        println!(
            "\nPhoto {}/{}: {}",
            i + 1,
            response.photos.len(),
            photo.photo_guid
        );

        // Create a custom filename if a caption exists
        let custom_filename = photo
            .caption
            .as_ref()
            .map(|caption| format!("{}_{}", i + 1, sanitize_filename(caption)));

        // Use the helper function to download the photo with correct MIME type detection
        match download_photo(photo, Some(i), download_dir, custom_filename).await {
            Ok(filepath) => {
                println!("  Saved to: {}", filepath);
            }
            Err(e) => {
                println!("  Failed to download: {}", e);
            }
        }
    }

    println!("\nDownload complete!");

    Ok(())
}
