# icloud-album-rs

A Rust library for interacting with iCloud shared albums.

## Overview

This library allows you to fetch metadata and photos from iCloud shared albums using a token. It provides a simple, async API to:

- Parse iCloud shared album tokens
- Handle Apple's redirect responses
- Fetch album metadata and photo information
- Retrieve photo asset URLs
- Enrich photos with their download URLs

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
icloud-album-rs = "0.1.0"
tokio = { version = "1", features = ["rt", "rt-multi-thread", "macros"] }
```

## Usage

### Basic Example

```rust
use icloud_album_rs::get_icloud_photos;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Your iCloud shared album token
    let token = "your_shared_album_token";
    
    // Fetch photos and metadata
    let response = get_icloud_photos(token).await?;
    
    // Print album info
    println!("Album: {}", response.metadata.stream_name);
    println!("Owner: {} {}", 
        response.metadata.user_first_name,
        response.metadata.user_last_name
    );
    println!("Photos: {}", response.photos.len());
    
    // Print information about each photo
    for (i, photo) in response.photos.iter().enumerate() {
        println!("Photo {}: {}", i + 1, photo.photo_guid);
        
        // Print the URL for each derivative
        for (key, derivative) in &photo.derivatives {
            if let Some(url) = &derivative.url {
                println!("  Derivative {}: {} ({}x{})", 
                    key, url, derivative.width.unwrap_or(0), derivative.height.unwrap_or(0));
            }
        }
    }
    
    Ok(())
}
```

### Data Structures

The main response type is `ICloudResponse` which contains:

- `metadata`: Information about the album (name, owner, etc.)
- `photos`: A vector of `Image` objects

Each `Image` contains:

- `photo_guid`: A unique identifier for the photo
- `derivatives`: A map of derivative identifiers to `Derivative` objects
- Additional metadata (caption, creation date, dimensions)

Each `Derivative` contains:

- `checksum`: A unique identifier for the derivative
- `file_size`: Size in bytes
- `width`, `height`: Dimensions in pixels
- `url`: The download URL for the derivative

## How it Works

1. The library generates a base URL from the token
2. It handles any redirects from the iCloud API
3. It fetches album metadata and photo information
4. It fetches URLs for all photo derivatives
5. It enriches the photos with their URLs

## Features

- Fully async API using Tokio
- Error handling and proper typing
- JSON serialization/deserialization using Serde

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.