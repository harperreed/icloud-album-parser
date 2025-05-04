use icloud_album_rs::redirect::get_redirected_base_url;
use reqwest::Client;
use serde_json::json;

// This example file demonstrates how to run redirect tests with mockito
// Run with: cargo run --example redirect_tests

// We'll use tokio::main but configure it to use a multi-threaded runtime
// to avoid conflicts with mockito, which expects its own runtime
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    println!("Running redirect tests...");

    let no_redirect_test_passed = test_no_redirect().await;
    if no_redirect_test_passed {
        println!("✅ No redirect test passed!");
    } else {
        println!("❌ No redirect test failed!");
    }

    let redirect_test_passed = test_with_redirect().await;
    if redirect_test_passed {
        println!("✅ Redirect test passed!");
    } else {
        println!("❌ Redirect test failed!");
    }

    let missing_host_test_passed = test_missing_host().await;
    if missing_host_test_passed {
        println!("✅ Missing host test passed!");
    } else {
        println!("❌ Missing host test failed!");
    }

    if no_redirect_test_passed && redirect_test_passed && missing_host_test_passed {
        println!("✅ All redirect tests passed!");
    } else {
        println!("❌ Some tests failed!");
        std::process::exit(1);
    }
}

async fn test_no_redirect() -> bool {
    println!("Testing no redirect case...");

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
    match get_redirected_base_url(&client, &base_url, token).await {
        Ok(result) => {
            let expected = base_url;
            let test_passed = result == expected;

            if !test_passed {
                println!("  ❌ Expected '{}', got '{}'", expected, result);
            } else {
                println!("  ✅ No redirect: URL unchanged as expected");
            }

            // Verify the mock was called
            let mock_called = mock.matched();
            if !mock_called {
                println!("  ❌ Mock was not called");
            }

            test_passed && mock_called
        }
        Err(e) => {
            println!("  ❌ Function returned error: {:?}", e);
            false
        }
    }
}

async fn test_with_redirect() -> bool {
    println!("Testing redirect case...");

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
    match get_redirected_base_url(&client, &base_url, token).await {
        Ok(result) => {
            let expected = format!(
                "https://p42-sharedstreams.icloud.com/{}/sharedstreams/",
                token
            );
            let test_passed = result == expected;

            if !test_passed {
                println!("  ❌ Expected '{}', got '{}'", expected, result);
            } else {
                println!("  ✅ Redirect: URL correctly transformed");
            }

            // Verify the mock was called
            let mock_called = mock.matched();
            if !mock_called {
                println!("  ❌ Mock was not called");
            }

            test_passed && mock_called
        }
        Err(e) => {
            println!("  ❌ Function returned error: {:?}", e);
            false
        }
    }
}

async fn test_missing_host() -> bool {
    println!("Testing redirect with missing host...");

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
    match get_redirected_base_url(&client, &base_url, token).await {
        Ok(result) => {
            let expected = base_url;
            let test_passed = result == expected;

            if !test_passed {
                println!("  ❌ Expected '{}', got '{}'", expected, result);
            } else {
                println!("  ✅ Missing host: Defaulted to original URL as expected");
            }

            // Verify the mock was called
            let mock_called = mock.matched();
            if !mock_called {
                println!("  ❌ Mock was not called");
            }

            test_passed && mock_called
        }
        Err(e) => {
            println!("  ❌ Function returned error: {:?}", e);
            false
        }
    }
}
