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
icloud-album-rs = "0.4.0"
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

### Examples

The library includes several examples in the `examples/` directory:

- `fetch_album.rs`: Basic example showing how to fetch and display album information
- `album_info.rs`: More detailed album metadata display with pretty formatting
- `download_photos.rs`: Shows how to download photos from an album to your local machine

#### Running the Examples

All examples require an iCloud shared album token. This is the string that appears after the URL when you share an album (e.g., `https://share.icloud.com/photos/abc1234defg5678`).

1. **Fetch and display an album:**

```bash
# Display detailed album information including all photos and their URLs
cargo run --example fetch_album -- "your_shared_album_token"
```

2. **Show album information with pretty formatting:**

```bash
# Display formatted album information and a summary of the first 5 photos
cargo run --example album_info -- "your_shared_album_token"

# With logging enabled (for debugging)
RUST_LOG=debug cargo run --example album_info -- "your_shared_album_token"
```

3. **Download photos from an album:**

```bash
# Download all photos from the album to the specified directory
cargo run --example download_photos -- "your_shared_album_token" "./download_dir"
```

The download example will:
- Create the download directory if it doesn't exist
- Find the highest resolution version of each photo
- Safely sanitize filenames based on photo captions
- Download all photos to the specified directory

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
- `file_size`: Size in bytes (can be string or number in API)
- `width`, `height`: Dimensions in pixels (can be string or number in API)
- `url`: The download URL for the derivative

## How it Works

1. The library generates a base URL from the token
2. It handles any redirects from the iCloud API
3. It fetches album metadata and photo information
4. It fetches URLs for all photo derivatives
5. It enriches the photos with their URLs

## Features

- Fully async API using Tokio
- Robust error handling with graceful degradation
- Flexible API response parsing that handles Apple's inconsistent data formats
- JSON serialization/deserialization using Serde
- Retry logic for intermittent API failures with configurable backoff strategies
- Comprehensive test suite including real-world integration tests
- Integrated logging system using the `log` crate
- Detailed schema validation for API responses

## Testing

The library includes a comprehensive test suite:

### Regular Unit Tests

Run the basic unit tests with:

```bash
cargo test
```

Some tests are marked with `#[ignore]` to avoid runtime conflicts or external API calls.

### All Tests (Including Ignored)

To run all tests including those marked with `#[ignore]`:

```bash
./scripts/run_all_tests.sh
```

This script runs all tests, including those with separate tokio runtimes, in a CI-friendly way. It handles tests that require separate tokio runtimes by:
1. Running all regular tests with `cargo test`
2. Running a special test runner for ignored tests
3. Reporting which tests need manual verification due to runtime conflicts

Alternatively, you can use the unified test runner directly:

```bash
cargo test --test run_all_tests
```

The unified test runner uses a sophisticated detection system to:
- Automatically detect tokio runtime conflicts
- Report which tests need manual verification
- Provide commands for running individual ignored tests
- Generate a detailed summary of test results

### Static Tests

To run tests that don't require HTTP mocking:

```bash
cargo run --example static_tests
```

These tests verify the core functionality of the library using static JSON responses without making HTTP requests or using mockito.

### Mockito Tests

To run all tests that use mockito:

```bash
cargo run --example mockito_standalone
```

This standalone launcher runs all the mockito tests in separate processes to avoid runtime conflicts. It includes:
- API response tests
- Asset URL tests
- Redirect tests
- Integration tests

### Individual Test Categories

You can also run individual test categories directly:

```bash
# Run API tests directly
cargo test --test api_test -- --ignored

# Run redirect tests directly
cargo test --test redirect_test -- --ignored

# Run integration tests directly
cargo test --test integration_test -- --ignored
```

### Real-World Integration Tests

To run tests against the actual iCloud API (requires internet connection):

```bash
cargo test --test real_world_test -- --ignored --nocapture
```

> **Note**: This makes real API calls to Apple's servers.

## Logging

The library uses the [`log`](https://crates.io/crates/log) crate for logging. You can enable and configure logging in your application:

```rust
// Initialize the env_logger (or any other logger implementation)
env_logger::init();

// Set log level via RUST_LOG environment variable
// Example: RUST_LOG=info,icloud_album_rs=debug cargo run
```

The library logs various events:
- **Warnings** for non-critical issues like API schema mismatches or type inconsistencies
- **Debug information** about API requests and responses
- **Error details** when operations fail

This is especially useful for diagnosing issues with the iCloud API and handling inconsistencies in responses.

## Handling API Quirks

The library includes several features to handle quirks in Apple's iCloud API:

- **Mixed Data Types**: Apple sometimes returns numeric values as strings. The library handles both formats seamlessly.
- **API Limitations**: Handles 400 Bad Request responses gracefully, allowing partial functionality even when URL fetching fails.
- **Retry Logic**: Automatically retries failed requests with configurable backoff strategies, including exponential backoff with jitter.
- **Schema Validation**: Verifies API responses against expected schemas and provides detailed reporting on inconsistencies.

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.