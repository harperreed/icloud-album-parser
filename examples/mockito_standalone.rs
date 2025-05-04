use std::process::Command;

// This example is a standalone launcher for tests that use mockito
// Run with: cargo run --example mockito_standalone

fn main() {
    println!("Running standalone mockito tests...");

    // Each function below spawns a separate process to run the tests
    // This avoids runtime conflicts by using separate processes

    run_test("api_response", run_api_test());
    run_test("asset_urls", run_asset_urls_test());
    run_test("redirect", run_redirect_test());
    run_test("integration", run_integration_test());

    println!("\nAll test processes completed!");
}

fn run_test(name: &str, success: bool) {
    println!("\n{} Test: {}", if success { "✅" } else { "❌" }, name);
}

fn run_api_test() -> bool {
    println!("Running API response tests...");

    // Run cargo test directly, but with the ignored test flag to run our async tests
    let output = Command::new("cargo")
        .args([
            "test",
            "--test",
            "api_test",
            "--",
            "--ignored",
            "test_api_response",
        ])
        .output()
        .expect("Failed to execute test command");

    let success = output.status.success();

    if success {
        println!("API response test succeeded");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("API response test failed: {}", stderr);
    }

    // Run the asset URLs test too
    let output = Command::new("cargo")
        .args([
            "test",
            "--test",
            "api_test",
            "--",
            "--ignored",
            "test_asset_urls",
        ])
        .output()
        .expect("Failed to execute test command");

    let asset_success = output.status.success();

    if asset_success {
        println!("Asset URLs test succeeded");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Asset URLs test failed: {}", stderr);
    }

    success && asset_success
}

fn run_asset_urls_test() -> bool {
    println!("Running asset URLs tests...");

    // Run cargo test directly
    let output = Command::new("cargo")
        .args([
            "test",
            "--test",
            "api_test",
            "--",
            "--ignored",
            "test_asset_urls",
        ])
        .output()
        .expect("Failed to execute test command");

    let success = output.status.success();

    if success {
        println!("Asset URLs test succeeded");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Asset URLs test failed: {}", stderr);
    }

    success
}

fn run_redirect_test() -> bool {
    println!("Running redirect tests...");

    // Run cargo test directly for each redirect test
    let tests = vec![
        "test_no_redirect",
        "test_with_redirect",
        "test_missing_host",
    ];

    let mut all_success = true;

    for test in tests {
        let output = Command::new("cargo")
            .args(["test", "--test", "redirect_test", "--", "--ignored", test])
            .output()
            .expect("Failed to execute test command");

        let success = output.status.success();

        if success {
            println!("{} succeeded", test);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("{} failed: {}", test, stderr);
            all_success = false;
        }
    }

    all_success
}

fn run_integration_test() -> bool {
    println!("Running integration tests...");

    // Run cargo test directly
    let output = Command::new("cargo")
        .args([
            "test",
            "--test",
            "integration_test",
            "--",
            "--ignored",
            "test_icloud_photos",
        ])
        .output()
        .expect("Failed to execute test command");

    let success = output.status.success();

    if success {
        println!("Integration test succeeded");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Integration test failed: {}", stderr);
    }

    success
}
