//! Example of downloading photos from an iCloud shared album
//!
//! Run with:
//! ```
//! cargo run --example download_photos -- "your_shared_album_token" "./download_dir"
//! ```

use icloud_album_rs::get_icloud_photos;
use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::copy;
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

    // Create a client for downloading
    let client = reqwest::Client::new();

    // Download each photo
    for (i, photo) in response.photos.iter().enumerate() {
        println!(
            "\nPhoto {}/{}: {}",
            i + 1,
            response.photos.len(),
            photo.photo_guid
        );

        // Find the highest resolution derivative
        let mut best_derivative = None;
        let mut max_resolution = 0;

        for (key, derivative) in &photo.derivatives {
            if let (Some(width), Some(height)) = (derivative.width, derivative.height) {
                let resolution = width * height;
                if resolution > max_resolution {
                    if let Some(url) = &derivative.url {
                        max_resolution = resolution;
                        best_derivative = Some((key, derivative, url.clone()));
                    }
                }
            }
        }

        if let Some((key, derivative, url)) = best_derivative {
            println!(
                "  Downloading derivative {} ({}x{})",
                key,
                derivative.width.unwrap_or(0),
                derivative.height.unwrap_or(0)
            );

            // Determine filename with proper sanitization
            let filename = if let Some(caption) = &photo.caption {
                format!(
                    "{}_{}_{}.jpg",
                    i + 1,
                    photo.photo_guid,
                    sanitize_filename(caption)
                )
            } else {
                format!("{}_{}.jpg", i + 1, photo.photo_guid)
            };

            let filepath = format!("{}/{}", download_dir, filename);

            // Download the file
            let response = client.get(&url).send().await?;
            let mut file = File::create(&filepath)?;
            let content = response.bytes().await?;
            copy(&mut content.as_ref(), &mut file)?;

            println!("  Saved to: {}", filepath);
        } else {
            println!("  No URL available for download");
        }
    }

    println!("\nDownload complete!");

    Ok(())
}
