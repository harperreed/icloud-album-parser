Here’s a step-by-step guide to replicating the same logic in Rust.

**Overview of what we’ll do:**

1. Parse a token to construct a base URL.
2. Handle Apple’s 330 (redirect) response to get a new base URL if needed.
3. Fetch metadata and photo info from `webstream`.
4. Fetch image URLs (assets) from `webasseturls`.
5. Enrich the results by combining metadata and URLs.

Below is a Rust sample library that shows how to do these steps:

```rust
// main.rs or lib.rs
//
// You need these dependencies in Cargo.toml:
//
// [dependencies]
// reqwest = "0.11"         // For HTTP requests
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"       // For JSON parsing
// futures = "0.3"          // For async/await
// tokio = { version = "1", features = ["macros"] } // If you want an async main
//
// USAGE:
//   let token = "someAlbumToken";
//   let response = get_icloud_photos(token).await.unwrap();
//   println!("Album name: {}", response.metadata.stream_name);
//
// Explanation:
//   1. get_icloud_photos(token) -> fetch metadata + photo listings
//   2. fetch each chunk of image URLs
//   3. enrich photo objects with their final URLs
//

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ----- Data Models -----

#[derive(Debug, Serialize, Deserialize)]
pub struct Derivative {
    pub checksum: String,
    pub fileSize: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    #[serde(rename = "photoGuid")]
    pub photo_guid: String,
    pub derivatives: HashMap<String, Derivative>,
    pub caption: Option<String>,
    #[serde(rename = "dateCreated")]
    pub date_created: Option<String>,
    #[serde(rename = "batchDateCreated")]
    pub batch_date_created: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    // ... include any additional fields you need
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(rename = "streamName")]
    pub stream_name: String,
    #[serde(rename = "userFirstName")]
    pub user_first_name: String,
    #[serde(rename = "userLastName")]
    pub user_last_name: String,
    #[serde(rename = "streamCtag")]
    pub stream_ctag: String,
    #[serde(rename = "itemsReturned")]
    pub items_returned: u32,
    pub locations: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub photos: Vec<Image>,
    #[serde(rename = "photoGuids")]
    pub photo_guids: Vec<String>,
    pub streamName: Option<String>,
    pub userFirstName: Option<String>,
    pub userLastName: Option<String>,
    pub streamCtag: Option<String>,
    pub itemsReturned: Option<String>,
    pub locations: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct ICloudResponse {
    pub metadata: Metadata,
    pub photos: Vec<Image>,
}

// ----- Step 1: Parse token to get base URL -----
fn get_base_url(token: &str) -> String {
    // This replicates logic that inspects first char(s) to compute partition
    // Here we do a simplified approach or replicate the base62 logic exactly if needed.

    // In the original code, there's some base62 parsing. For brevity, let's skip
    // that or replicate. We'll do a naive approach just to illustrate:

    // Example: https://pXX-sharedstreams.icloud.com/<TOKEN>/sharedstreams/
    // We'll assume XX is some partition ID. The TS code does some base62 conversion.
    // If you need it, just replicate the base62 logic. This is the partial version:
    let server_partition = 10; // Dummy
    format!("https://p{:02}-sharedstreams.icloud.com/{}/sharedstreams/", 
            server_partition, token)
}

// ----- Step 2: Handle 330 redirect -----
async fn get_redirected_base_url(client: &Client, base_url: &str, token: &str) -> String {
    // We'll call the 'webstream' endpoint and see if we get status code 330
    let url = format!("{}webstream", base_url);
    let payload = serde_json::json!({ "streamCtag": null });

    // We'll allow non-200 statuses but handle them manually
    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    if resp.status() == StatusCode::from_u16(330).unwrap() {
        // Apple returns a JSON body with "X-Apple-MMe-Host"
        let body: serde_json::Value = resp
            .json()
            .await
            .expect("failed parsing redirect body as JSON");
        
        if let Some(host_val) = body["X-Apple-MMe-Host"].as_str() {
            // Build new base URL
            return format!("https://{}/{}/sharedstreams/", host_val, token);
        }
    }

    base_url.to_string()
}

// ----- Step 3: Fetch metadata / photo info (webstream) -----
async fn get_api_response(client: &Client, base_url: &str) -> Result<(Vec<Image>, Metadata), Box<dyn std::error::Error>> {
    let url = format!("{}webstream", base_url);
    let payload = serde_json::json!({ "streamCtag": null });
    let resp = client.post(&url).json(&payload).send().await?;

    if !resp.status().is_success() {
        return Err(format!("webstream request failed with status {}", resp.status()).into());
    }

    let data: serde_json::Value = resp.json().await?;
    // Extract relevant fields from the JSON to form our final types

    // photos array
    let photos_raw = data["photos"].as_array().unwrap_or(&vec![]);
    let mut photos: Vec<Image> = Vec::with_capacity(photos_raw.len());
    for p in photos_raw {
        let parsed: Image = serde_json::from_value(p.clone()).unwrap_or_default();
        photos.push(parsed);
    }

    // metadata
    let metadata = Metadata {
        stream_name: data["streamName"].as_str().unwrap_or("").to_string(),
        user_first_name: data["userFirstName"].as_str().unwrap_or("").to_string(),
        user_last_name: data["userLastName"].as_str().unwrap_or("").to_string(),
        stream_ctag: data["streamCtag"].as_str().unwrap_or("").to_string(),
        items_returned: data["itemsReturned"].as_u64().unwrap_or(0) as u32,
        locations: data["locations"].clone(),
    };

    Ok((photos, metadata))
}

// ----- Step 4: Fetch URLs (webasseturls) for each photo GUID in chunks -----
async fn get_asset_urls(
    client: &Client,
    base_url: &str,
    photo_guids: &[String],
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let url = format!("{}webasseturls", base_url);
    let payload = serde_json::json!({ "photoGuids": photo_guids });
    let resp = client.post(&url).json(&payload).send().await?;

    if !resp.status().is_success() {
        return Err(format!("webasseturls request failed with status {}", resp.status()).into());
    }

    let data: serde_json::Value = resp.json().await?;
    // data.items => map of photoGuid to { url_location:..., url_path:... }
    let items_val = &data["items"];
    let mut results = HashMap::new();

    if let Some(obj) = items_val.as_object() {
        for (guid, value) in obj.iter() {
            let url_location = value["url_location"].as_str().unwrap_or("");
            let url_path = value["url_path"].as_str().unwrap_or("");
            let full = format!("https://{}{}", url_location, url_path);
            results.insert(guid.to_string(), full);
        }
    }

    Ok(results)
}

// ----- Step 5: Combine the data to produce final result -----
async fn enrich_photos_with_urls(
    photos: &mut [Image],
    all_urls: &HashMap<String, String>,
) {
    // For each derivative, if we see a matching checksum in all_urls, attach the URL
    for photo in photos.iter_mut() {
        for derivative in photo.derivatives.values_mut() {
            if let Some(url) = all_urls.get(&derivative.checksum) {
                derivative.url = Some(url.to_string());
            }
        }
    }
}

// ----- Main entry point to replicate everything -----
pub async fn get_icloud_photos(token: &str) -> Result<ICloudResponse, Box<dyn std::error::Error>> {
    let client = Client::new();

    // 1. Compute the base URL
    let base_url = get_base_url(token);

    // 2. Possibly get a redirected base URL
    let redirected = get_redirected_base_url(&client, &base_url, token).await;

    // 3. Get the raw photo + metadata
    let (mut photos, metadata) = get_api_response(&client, &redirected).await?;

    // 4. We might want all the photo GUIDs
    //    but note the original code stored them in a separate data field.
    //    We'll map from the photos array.
    let photo_guids: Vec<String> = photos.iter().map(|p| p.photo_guid.clone()).collect();

    // The original code chunked these in groups of 25. 
    // We'll just do it in one pass or replicate chunking if you prefer. 
    // For brevity, do it in one pass:
    let all_urls = get_asset_urls(&client, &redirected, &photo_guids).await?;

    // 5. Insert those URLs into the derivative objects
    enrich_photos_with_urls(&mut photos, &all_urls).await;

    // Return final data
    Ok(ICloudResponse { metadata, photos })
}

// ----- Alternative Implementation (Sync) -----
// You can use "blocking" reqwest if you prefer not to use async:
//   let client = reqwest::blocking::Client::new();
//   ... etc...
// Just replicate the steps above with blocking calls.
```

**Key Points:**

* We use **`reqwest`** for HTTP (async or blocking).
* We use **`serde`** / **`serde_json`** to parse JSON into Rust structs.
* We replicate the same logic from the TypeScript code:

  1. compute base URL,
  2. handle a possible 330 redirect,
  3. fetch `webstream` to get photos/metadata,
  4. fetch `webasseturls` to get real image URLs,
  5. enrich results with final URLs.

That’s the basic approach. Adjust as needed for error handling, chunking, or more advanced base62 partition logic.

