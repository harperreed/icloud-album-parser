//! Example of fetching an iCloud shared album
//! 
//! Run with:
//! ```
//! cargo run --example fetch_album -- "your_shared_album_token"
//! ```

use icloud_album_rs::get_icloud_photos;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the token from the command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run --example fetch_album -- \"your_shared_album_token\"");
        std::process::exit(1);
    }
    
    let token = &args[1];
    
    println!("Fetching shared album with token: {}", token);
    
    // Fetch photos and metadata
    let response = get_icloud_photos(token).await?;
    
    // Print album info
    println!("\nAlbum: {}", response.metadata.stream_name);
    println!("Owner: {} {}", 
        response.metadata.user_first_name,
        response.metadata.user_last_name
    );
    println!("Photos: {}", response.photos.len());
    
    // Print information about each photo
    for (i, photo) in response.photos.iter().enumerate() {
        println!("\nPhoto {}: {}", i + 1, photo.photo_guid);
        
        if let Some(caption) = &photo.caption {
            println!("  Caption: {}", caption);
        }
        
        if let Some(date) = &photo.date_created {
            println!("  Date: {}", date);
        }
        
        println!("  Dimensions: {}x{}", 
            photo.width.unwrap_or(0), 
            photo.height.unwrap_or(0)
        );
        
        // Print the URL for each derivative
        println!("  Derivatives:");
        for (key, derivative) in &photo.derivatives {
            if let Some(url) = &derivative.url {
                println!("    {}: {}x{} - {}", 
                    key, 
                    derivative.width.unwrap_or(0),
                    derivative.height.unwrap_or(0),
                    url
                );
            } else {
                println!("    {}: {}x{} - No URL", 
                    key,
                    derivative.width.unwrap_or(0),
                    derivative.height.unwrap_or(0)
                );
            }
        }
    }
    
    Ok(())
}