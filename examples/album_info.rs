//! Example of fetching basic information about an iCloud shared album
//! 
//! Run with:
//! ```
//! cargo run --example album_info -- "your_shared_album_token"
//! ```

use icloud_album_rs::get_icloud_photos;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the token from the command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run --example album_info -- \"your_shared_album_token\"");
        std::process::exit(1);
    }
    
    let token = &args[1];
    
    println!("Fetching album info for token: {}", token);
    
    // Fetch photos and metadata
    let response = get_icloud_photos(token).await?;
    
    // Print a table with album info
    println!("\n┌─────────────────────────────────────┐");
    println!("│ Album Information                   │");
    println!("├─────────────────┬───────────────────┤");
    println!("│ Name            │ {:<17} │", response.metadata.stream_name);
    println!("│ Owner           │ {:<17} │", 
        format!("{} {}", 
            response.metadata.user_first_name,
            response.metadata.user_last_name
        )
    );
    println!("│ Photos          │ {:<17} │", response.photos.len());
    println!("│ Stream CTag     │ {:<17} │", response.metadata.stream_ctag);
    println!("│ Items Returned  │ {:<17} │", response.metadata.items_returned);
    println!("└─────────────────┴───────────────────┘");
    
    // Print a summary of the photos
    if !response.photos.is_empty() {
        println!("\n┌─────────────────────────────────────┐");
        println!("│ Photo Summary                       │");
        println!("├────┬──────────────┬─────────────────┤");
        println!("│ #  │ Date Created │ Caption         │");
        println!("├────┼──────────────┼─────────────────┤");
        
        for (i, photo) in response.photos.iter().take(5).enumerate() {
            let date = photo.date_created.as_deref().unwrap_or("N/A");
            let caption = photo.caption.as_deref().unwrap_or("N/A");
            println!("│ {:<2} │ {:<12} │ {:<15} │", i + 1, date, caption);
        }
        
        if response.photos.len() > 5 {
            println!("│ .. │ ............ │ ............. │");
        }
        
        println!("└────┴──────────────┴─────────────────┘");
    }
    
    Ok(())
}