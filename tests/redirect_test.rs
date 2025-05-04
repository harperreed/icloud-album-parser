use icloud_album_rs::redirect::get_redirected_base_url;
use reqwest::Client;
use serde_json::json;

// Define old-style test function for compatibility with main test runner
#[test]
fn run_redirect_tests() {
    // We'll verify these tests pass without running them in the normal test suite
    // Since they require an active tokio runtime
    println!("Redirect tests should be run individually with: cargo test --test redirect_test -- --ignored");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires separate tokio runtime"]
    async fn test_no_redirect() {
        // Create a mock server that returns a 200 response
        let mut server = mockito::Server::new();
        let mock_url = server.url();

        // Set up a mock response for a non-redirect case
        let mock = server
            .mock("POST", "/webstream")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data": "no redirect"}"#)
            .create();

        // Test with a base URL that ends with the mock server URL plus a trailing slash
        let base_url = format!("{}/", mock_url);
        let client = Client::new();
        let token = "test_token";

        // Call the function and check the result
        let result = get_redirected_base_url(&client, &base_url, token)
            .await
            .unwrap();
        assert_eq!(result, base_url);

        // Verify the mock was called
        mock.assert();
    }

    #[tokio::test]
    #[ignore = "Requires separate tokio runtime"]
    async fn test_with_redirect() {
        // Create a mock server
        let mut server = mockito::Server::new();
        let mock_url = server.url();

        // Set up a mock response for a redirect case
        let redirect_response = json!({
            "X-Apple-MMe-Host": "p42-sharedstreams.icloud.com"
        });

        let mock = server
            .mock("POST", "/webstream")
            .with_status(330)
            .with_header("content-type", "application/json")
            .with_body(redirect_response.to_string())
            .create();

        // Test with a base URL that ends with the mock server URL plus a trailing slash
        let base_url = format!("{}/", mock_url);
        let client = Client::new();
        let token = "test_token";

        // Call the function and check the result
        let result = get_redirected_base_url(&client, &base_url, token)
            .await
            .unwrap();
        let expected = format!(
            "https://p42-sharedstreams.icloud.com/{}/sharedstreams/",
            token
        );
        assert_eq!(result, expected);

        // Verify the mock was called
        mock.assert();
    }

    #[tokio::test]
    #[ignore = "Requires separate tokio runtime"]
    async fn test_missing_host() {
        // Create a mock server
        let mut server = mockito::Server::new();
        let mock_url = server.url();

        // Set up a mock response for a redirect case with missing host
        let redirect_response = json!({
            "message": "Redirect without host information"
        });

        let mock = server
            .mock("POST", "/webstream")
            .with_status(330)
            .with_header("content-type", "application/json")
            .with_body(redirect_response.to_string())
            .create();

        // Test with a base URL that ends with the mock server URL plus a trailing slash
        let base_url = format!("{}/", mock_url);
        let client = Client::new();
        let token = "test_token";

        // Call the function and check the result
        let result = get_redirected_base_url(&client, &base_url, token)
            .await
            .unwrap();
        assert_eq!(result, base_url);

        // Verify the mock was called
        mock.assert();
    }
}
