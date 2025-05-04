use icloud_album_rs::redirect::get_redirected_base_url;
use reqwest::Client;
use serde_json::json;

// We'll use a main function with #[tokio::main] to run tests
// This approach works better when using mockito
#[tokio::main]
async fn main() {
    // Run all tests
    let no_redirect_success = test_redirect_handling_no_redirect().await;
    assert!(no_redirect_success, "No redirect test failed");
    
    let with_redirect_success = test_redirect_handling_with_redirect().await;
    assert!(with_redirect_success, "With redirect test failed");
    
    let missing_host_success = test_redirect_handling_missing_host().await;
    assert!(missing_host_success, "Missing host test failed");
}

async fn test_redirect_handling_no_redirect() -> bool {
    // Create a mock server that returns a 200 response
    let mut mock_server = mockito::Server::new();
    let mock_url = mock_server.url();
    
    // Set up a mock response for a non-redirect case
    let _m = mock_server.mock("POST", "/webstream")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"data": "no redirect"}"#)
        .create();
    
    // Test with a base URL that ends with the mock server URL plus a trailing slash
    let base_url = format!("{}/", mock_url);
    let client = Client::new();
    let token = "test_token";
    
    // Call the function and check the result
    match get_redirected_base_url(&client, &base_url, token).await {
        Ok(result) => result == base_url,
        Err(_) => false
    }
}

async fn test_redirect_handling_with_redirect() -> bool {
    // Create a mock server
    let mut mock_server = mockito::Server::new();
    let mock_url = mock_server.url();
    
    // Set up a mock response for a redirect case
    let redirect_response = json!({
        "X-Apple-MMe-Host": "p42-sharedstreams.icloud.com"
    });
    
    let _m = mock_server.mock("POST", "/webstream")
        .with_status(330)
        .with_header("content-type", "application/json")
        .with_body(redirect_response.to_string())
        .create();
    
    // Test with a base URL that ends with the mock server URL plus a trailing slash
    let base_url = format!("{}/", mock_url);
    let client = Client::new();
    let token = "test_token";
    
    // Call the function and check the result
    match get_redirected_base_url(&client, &base_url, token).await {
        Ok(result) => {
            let expected = format!("https://p42-sharedstreams.icloud.com/{}/sharedstreams/", token);
            result == expected
        },
        Err(_) => false
    }
}

async fn test_redirect_handling_missing_host() -> bool {
    // Create a mock server
    let mut mock_server = mockito::Server::new();
    let mock_url = mock_server.url();
    
    // Set up a mock response for a redirect case with missing host
    let redirect_response = json!({
        "message": "Redirect without host information"
    });
    
    let _m = mock_server.mock("POST", "/webstream")
        .with_status(330)
        .with_header("content-type", "application/json")
        .with_body(redirect_response.to_string())
        .create();
    
    // Test with a base URL that ends with the mock server URL plus a trailing slash
    let base_url = format!("{}/", mock_url);
    let client = Client::new();
    let token = "test_token";
    
    // Call the function and check the result
    match get_redirected_base_url(&client, &base_url, token).await {
        Ok(result) => result == base_url,
        Err(_) => false
    }
}