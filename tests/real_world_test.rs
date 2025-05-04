//! Real-world integration test using an actual iCloud shared album token
//! 
//! This test uses a real shared album token to verify the library against iCloud's API.
//! It's meant to be run manually rather than as part of automated CI testing.
//! 
//! To run this test:
//! ```
//! cargo test --test real_world_test -- --nocapture
//! ```

use icloud_album_rs::get_icloud_photos;

#[cfg(test)]
mod tests {
    use super::*;
    
    // The actual test function that will be recognized by the test runner
    #[tokio::test]
    async fn test_real_icloud_album() {
        println!("Running real-world integration test with actual iCloud shared album...");
        println!("Note: This test depends on an external service and may fail if the service changes.");
        
        let result = test_real_album().await;
        
        match result {
            Ok(_) => println!("✅ Real-world test passed!"),
            Err(e) => println!("❌ Real-world test failed: {}", e),
        }
        
        // Assert success so the test passes
        if let Err(e) = &result {
            panic!("Test failed: {}", e);
        }
    }
}

async fn test_real_album() -> Result<(), Box<dyn std::error::Error>> {
    // Real shared album token provided by user
    let token = "B2T5VaUrzMLxwU";
    
    println!("\nFetching album with token: {}", token);
    
    // Fetch the album using our library
    let response = get_icloud_photos(token).await?;
    
    // Print album info
    println!("\n📱 Album: {}", response.metadata.stream_name);
    println!("👤 Owner: {} {}", 
        response.metadata.user_first_name,
        response.metadata.user_last_name
    );
    println!("🖼️ Photo count: {}", response.photos.len());
    
    // Make sure we got some photos
    if response.photos.is_empty() {
        return Err("Album contains no photos".into());
    }
    
    // Check if photos have derivatives
    for (i, photo) in response.photos.iter().enumerate().take(5) {
        println!("\n📷 Photo {}: {}", i + 1, photo.photo_guid);
        
        if photo.derivatives.is_empty() {
            return Err(format!("Photo {} has no derivatives", i + 1).into());
        }
        
        // Check if at least one derivative has a URL
        let has_url = photo.derivatives.values().any(|d| d.url.is_some());
        if !has_url {
            return Err(format!("Photo {} has no derivatives with URLs", i + 1).into());
        }
        
        // Print some derivative info
        for (key, derivative) in photo.derivatives.iter().take(3) {
            if let Some(url) = &derivative.url {
                println!("  📌 Derivative {}: {}x{}", 
                    key, 
                    derivative.width.unwrap_or(0),
                    derivative.height.unwrap_or(0)
                );
                
                // Just print the beginning of the URL to avoid too much output
                let url_prefix = if url.len() > 60 {
                    format!("{}...", &url[0..60])
                } else {
                    url.clone()
                };
                println!("     🔗 URL: {}", url_prefix);
            }
        }
        
        if photo.derivatives.len() > 3 {
            println!("     ... and {} more derivatives", photo.derivatives.len() - 3);
        }
    }
    
    if response.photos.len() > 5 {
        println!("\n... and {} more photos", response.photos.len() - 5);
    }
    
    // If we made it this far, everything worked!
    println!("\n✨ Successfully accessed and parsed the iCloud shared album!");
    Ok(())
}