//! API client for interacting with iCloud shared albums.
//!
//! This module provides functions to fetch album metadata, photo information,
//! and asset URLs from the iCloud shared album API endpoints.

use crate::models::{Image, Metadata};
use log::warn;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Custom error type for API-related errors
#[derive(Debug)]
pub enum ApiError {
    /// Error from a network request
    NetworkError(reqwest::Error),
    /// Error when parsing JSON
    JsonParseError(String),
    /// Error when a field is missing in the response
    MissingFieldError(String),
    /// Error when a request fails with a status code
    RequestError {
        /// The HTTP status code
        status: Option<u16>,
        /// Error message
        message: String,
    },
    /// Error during retries
    RetryError(String),
    /// Other errors
    Other(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NetworkError(e) => write!(f, "Network error: {}", e),
            ApiError::JsonParseError(msg) => write!(f, "JSON parse error: {}", msg),
            ApiError::MissingFieldError(field) => write!(f, "Missing field in response: {}", field),
            ApiError::RequestError { status, message } => {
                if let Some(status_code) = status {
                    write!(f, "Request error (status {}): {}", status_code, message)
                } else {
                    write!(f, "Request error: {}", message)
                }
            }
            ApiError::RetryError(msg) => write!(f, "Retry error: {}", msg),
            ApiError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl Error for ApiError {}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::NetworkError(err)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::JsonParseError(err.to_string())
    }
}

impl From<String> for ApiError {
    fn from(err: String) -> Self {
        ApiError::Other(err)
    }
}

impl From<&str> for ApiError {
    fn from(err: &str) -> Self {
        ApiError::Other(err.to_string())
    }
}

// Don't need an explicit conversion from ApiError to Box<dyn Error>
// since this is provided by the standard library for any type that implements Error

/// Fetches metadata and photos from the iCloud API
///
/// This function makes a POST request to the webstream endpoint and extracts
/// the metadata and photos from the response.
///
/// # Arguments
///
/// * `client` - A reqwest HTTP client
/// * `base_url` - The base URL for API requests
///
/// # Returns
///
/// A tuple containing a vector of Images and Metadata information
pub async fn get_api_response(
    client: &Client,
    base_url: &str,
) -> Result<(Vec<Image>, Metadata), ApiError> {
    // Build the URL for the webstream endpoint
    let url = format!("{}webstream", base_url);

    // Create the payload with a null streamCtag
    let payload = json!({ "streamCtag": null });

    // Make the POST request
    let resp = client.post(&url).json(&payload).send().await?;

    // Check if the request was successful
    if !resp.status().is_success() {
        return Err(ApiError::RequestError {
            status: Some(resp.status().as_u16()),
            message: format!("webstream request failed"),
        });
    }

    // Parse the response as JSON
    let data: serde_json::Value = resp.json().await?;

    // Validate the API response against expected schema
    let issues = validate_api_schema(&data, "webstream");
    if !issues.is_empty() {
        // Log all validation issues as warnings
        for (field, failure) in &issues {
            match failure {
                ValidationFailure::Missing => {
                    log_warning(&format!("Schema validation: Missing field '{}'", field));
                }
                ValidationFailure::WrongType => {
                    log_warning(&format!(
                        "Schema validation: Field '{}' has wrong type",
                        field
                    ));
                }
                ValidationFailure::InvalidValue(msg) => {
                    log_warning(&format!(
                        "Schema validation: Field '{}' has invalid value: {}",
                        field, msg
                    ));
                }
            }
        }

        // If there are critical issues, we might want to fail the request
        // For now, we log but continue with the processing
        log_warning(&format!(
            "API response has {} schema validation issues",
            issues.len()
        ));
    }

    // Extract the photos array from the JSON
    let photos_raw = match data.get("photos") {
        Some(photos) => match photos.as_array() {
            Some(photos_array) => photos_array,
            None => {
                // Log warning but don't fail - photos field exists but is not an array
                log_warning("'photos' field is not an array");
                &Vec::new()
            }
        },
        None => {
            // Log warning but don't fail - missing photos field
            log_warning("Missing 'photos' field in API response");
            &Vec::new()
        }
    };

    let mut photos: Vec<Image> = Vec::with_capacity(photos_raw.len());

    // Parse each photo into an Image struct
    for (index, photo) in photos_raw.iter().enumerate() {
        match serde_json::from_value::<Image>(photo.clone()) {
            Ok(parsed) => photos.push(parsed),
            Err(e) => {
                // Log warning with more context but don't fail the entire request
                log_warning(&format!("Failed to parse photo at index {}: {}", index, e));
            }
        }
    }

    // Extract the metadata fields from the JSON with better error handling
    // streamName is considered required for a valid album
    let stream_name = get_string_field(
        &data,
        "streamName",
        "Unknown Album",
        FieldSeverity::Required,
    )?;
    // User info is helpful but not critical
    let user_first_name = get_string_field(&data, "userFirstName", "", FieldSeverity::Optional)?;
    let user_last_name = get_string_field(&data, "userLastName", "", FieldSeverity::Optional)?;
    // streamCtag is important for API contract but we can continue without it
    let stream_ctag = get_string_field(&data, "streamCtag", "", FieldSeverity::Optional)?;
    // itemsReturned is useful for validation but not essential
    let items_returned = get_u32_field(&data, "itemsReturned", 0, FieldSeverity::Optional)?;

    // For locations, we'll just clone whatever is there or use null if missing
    let locations = match data.get("locations") {
        Some(value) => value.clone(),
        None => {
            log_warning("Missing 'locations' field");
            serde_json::Value::Null
        }
    };

    let metadata = Metadata {
        stream_name,
        user_first_name,
        user_last_name,
        stream_ctag,
        items_returned,
        locations,
    };

    Ok((photos, metadata))
}

/// Severity level for field validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldSeverity {
    /// Field is required and must be of the correct type
    Required,
    /// Field is optional but must be of the correct type if present
    Optional,
    /// Field will be silently defaulted if missing or invalid
    Lenient,
}

/// Reason a field failed validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationFailure {
    /// Field is missing
    Missing,
    /// Field has the wrong type
    WrongType,
    /// Field value is invalid (e.g., out of range)
    InvalidValue(String),
}

/// Generic field extractor trait for working with JSON values
trait JsonFieldExtractor<T> {
    /// Extract a field from JSON with the given name and severity level
    fn extract(
        &self,
        value: &serde_json::Value,
        field_name: &str,
        severity: FieldSeverity,
    ) -> Result<T, ApiError>;

    /// Get a default value for this type
    fn default_value(&self) -> T;

    /// Handle the case where the field exists but has the wrong type
    fn handle_wrong_type(&self, field_name: &str, severity: FieldSeverity) -> Result<T, ApiError>;

    /// Handle the case where the field is missing
    fn handle_missing(&self, field_name: &str, severity: FieldSeverity) -> Result<T, ApiError>;
}

/// String field extractor implementation
struct StringFieldExtractor {
    default_value: String,
}

impl JsonFieldExtractor<String> for StringFieldExtractor {
    fn extract(
        &self,
        value: &serde_json::Value,
        field_name: &str,
        severity: FieldSeverity,
    ) -> Result<String, ApiError> {
        match value.get(field_name) {
            Some(value) => match value.as_str() {
                Some(s) => Ok(s.to_string()),
                None => self.handle_wrong_type(field_name, severity),
            },
            None => self.handle_missing(field_name, severity),
        }
    }

    fn default_value(&self) -> String {
        self.default_value.clone()
    }

    fn handle_wrong_type(
        &self,
        field_name: &str,
        severity: FieldSeverity,
    ) -> Result<String, ApiError> {
        let err_msg = format!("Field '{}' is not a string", field_name);
        match severity {
            FieldSeverity::Required => Err(ApiError::MissingFieldError(format!(
                "{} (required field with wrong type)",
                field_name
            ))),
            FieldSeverity::Optional | FieldSeverity::Lenient => {
                log_warning(&err_msg);
                Ok(self.default_value())
            }
        }
    }

    fn handle_missing(
        &self,
        field_name: &str,
        severity: FieldSeverity,
    ) -> Result<String, ApiError> {
        let err_msg = format!("Missing '{}' field", field_name);
        match severity {
            FieldSeverity::Required => Err(ApiError::MissingFieldError(field_name.to_string())),
            FieldSeverity::Optional | FieldSeverity::Lenient => {
                log_warning(&err_msg);
                Ok(self.default_value())
            }
        }
    }
}

/// U32 field extractor implementation
struct U32FieldExtractor {
    default_value: u32,
}

impl JsonFieldExtractor<u32> for U32FieldExtractor {
    fn extract(
        &self,
        value: &serde_json::Value,
        field_name: &str,
        severity: FieldSeverity,
    ) -> Result<u32, ApiError> {
        match value.get(field_name) {
            Some(value) => {
                // Try to parse as u64 first
                if let Some(n) = value.as_u64() {
                    if n <= u32::MAX as u64 {
                        return Ok(n as u32);
                    }

                    let err_msg = format!("Field '{}' is too large for u32", field_name);
                    match severity {
                        FieldSeverity::Required => {
                            return Err(ApiError::JsonParseError(err_msg));
                        }
                        _ => {
                            log_warning(&err_msg);
                            return Ok(self.default_value());
                        }
                    }
                }
                // Try to parse as string
                else if let Some(s) = value.as_str() {
                    match s.parse::<u32>() {
                        Ok(n) => return Ok(n),
                        Err(_) => {
                            let err_msg = format!("Failed to parse '{}' as u32", field_name);
                            match severity {
                                FieldSeverity::Required => {
                                    return Err(ApiError::JsonParseError(err_msg));
                                }
                                _ => {
                                    log_warning(&err_msg);
                                    return Ok(self.default_value());
                                }
                            }
                        }
                    }
                } else {
                    return self.handle_wrong_type(field_name, severity);
                }
            }
            None => self.handle_missing(field_name, severity),
        }
    }

    fn default_value(&self) -> u32 {
        self.default_value
    }

    fn handle_wrong_type(
        &self,
        field_name: &str,
        severity: FieldSeverity,
    ) -> Result<u32, ApiError> {
        let err_msg = format!("Field '{}' is neither a number nor a string", field_name);
        match severity {
            FieldSeverity::Required => Err(ApiError::MissingFieldError(format!(
                "{} (required field with wrong type)",
                field_name
            ))),
            _ => {
                log_warning(&err_msg);
                Ok(self.default_value())
            }
        }
    }

    fn handle_missing(&self, field_name: &str, severity: FieldSeverity) -> Result<u32, ApiError> {
        let err_msg = format!("Missing '{}' field", field_name);
        match severity {
            FieldSeverity::Required => Err(ApiError::MissingFieldError(field_name.to_string())),
            _ => {
                log_warning(&err_msg);
                Ok(self.default_value())
            }
        }
    }
}

/// Helper function to extract a string field from JSON with proper error handling
///
/// # Arguments
///
/// * `data` - The JSON data to extract from
/// * `field_name` - The name of the field to extract
/// * `default` - The default value to use if the field is missing or invalid
/// * `severity` - How strict to be about validation
///
/// # Returns
///
/// The extracted string value or an error if required field is missing/invalid
fn get_string_field(
    data: &serde_json::Value,
    field_name: &str,
    default: &str,
    severity: FieldSeverity,
) -> Result<String, ApiError> {
    let extractor = StringFieldExtractor {
        default_value: default.to_string(),
    };
    extractor.extract(data, field_name, severity)
}

/// Helper function to extract a u32 field from JSON with proper error handling
///
/// # Arguments
///
/// * `data` - The JSON data to extract from
/// * `field_name` - The name of the field to extract
/// * `default` - The default value to use if the field is missing or invalid
/// * `severity` - How strict to be about validation
///
/// # Returns
///
/// The extracted u32 value or an error if required field is missing/invalid
fn get_u32_field(
    data: &serde_json::Value,
    field_name: &str,
    default: u32,
    severity: FieldSeverity,
) -> Result<u32, ApiError> {
    let extractor = U32FieldExtractor {
        default_value: default,
    };
    extractor.extract(data, field_name, severity)
}

/// Helper function for logging warnings
///
/// Uses the `log` crate to log warnings which can be captured by any logger
/// implementation the user configures.
fn log_warning(message: &str) {
    warn!("{}", message);
}

/// Validates the API response against expected schema
///
/// This function checks if the API response conforms to the expected schema
/// and reports any missing or invalid fields.
///
/// # Arguments
///
/// * `data` - The JSON data to validate
/// * `schema_name` - The name of the schema being validated (for reporting)
///
/// # Returns
///
/// A vector of validation issues found (empty if valid)
pub fn validate_api_schema(
    data: &serde_json::Value,
    schema_name: &str,
) -> Vec<(String, ValidationFailure)> {
    let mut issues = Vec::new();

    match schema_name {
        "webstream" => {
            // Required fields
            check_field_exists(data, "streamName", &mut issues);
            check_field_exists(data, "streamCtag", &mut issues);

            // Array fields
            if let Some(photos) = data.get("photos") {
                if !photos.is_array() {
                    issues.push(("photos".to_string(), ValidationFailure::WrongType));
                } else if let Some(photos_arr) = photos.as_array() {
                    // Validate each photo in the array
                    for (i, photo) in photos_arr.iter().enumerate() {
                        let prefix = format!("photos[{}]", i);

                        // Each photo should have these fields
                        check_field_exists_with_prefix(photo, "photoGuid", &prefix, &mut issues);
                        check_field_exists_with_prefix(photo, "derivatives", &prefix, &mut issues);

                        // Check derivatives object
                        if let Some(derivatives) = photo.get("derivatives") {
                            if !derivatives.is_object() {
                                issues.push((
                                    format!("{}.derivatives", prefix),
                                    ValidationFailure::WrongType,
                                ));
                            }
                        }
                    }
                }
            } else {
                issues.push(("photos".to_string(), ValidationFailure::Missing));
            }
        }
        "webasseturls" => {
            // Required fields
            check_field_exists(data, "items", &mut issues);

            // Validate items object
            if let Some(items) = data.get("items") {
                if !items.is_object() {
                    issues.push(("items".to_string(), ValidationFailure::WrongType));
                } else if let Some(items_obj) = items.as_object() {
                    // Each item should have url_location and url_path
                    for (guid, item) in items_obj {
                        let prefix = format!("items.{}", guid);

                        check_field_exists_with_prefix(item, "url_location", &prefix, &mut issues);
                        check_field_exists_with_prefix(item, "url_path", &prefix, &mut issues);
                    }
                }
            }
        }
        _ => {
            log_warning(&format!("Unknown schema name: {}", schema_name));
        }
    }

    issues
}

/// Field validator trait for validating fields in JSON
trait JsonFieldValidator {
    /// Check if a field meets validation criteria
    fn validate(
        &self,
        data: &serde_json::Value,
        field: &str,
        field_path: &str,
        issues: &mut Vec<(String, ValidationFailure)>,
    );
}

/// Existence validator that checks if a field exists
struct FieldExistsValidator;

impl JsonFieldValidator for FieldExistsValidator {
    fn validate(
        &self,
        data: &serde_json::Value,
        field: &str,
        field_path: &str,
        issues: &mut Vec<(String, ValidationFailure)>,
    ) {
        if !data.get(field).is_some() {
            issues.push((field_path.to_string(), ValidationFailure::Missing));
        }
    }
}

// Helper to check if a field exists and add to issues if not
fn check_field_exists(
    data: &serde_json::Value,
    field: &str,
    issues: &mut Vec<(String, ValidationFailure)>,
) {
    let validator = FieldExistsValidator;
    validator.validate(data, field, field, issues);
}

// Helper to check if a field exists with a prefix and add to issues if not
fn check_field_exists_with_prefix(
    data: &serde_json::Value,
    field: &str,
    prefix: &str,
    issues: &mut Vec<(String, ValidationFailure)>,
) {
    let validator = FieldExistsValidator;
    let field_path = format!("{}.{}", prefix, field);
    validator.validate(data, field, &field_path, issues);
}

/// Backoff strategy for retries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackoffStrategy {
    /// No backoff - constant delay between retries
    Constant,
    /// Linear backoff - delay increases linearly with retry attempt
    Linear,
    /// Exponential backoff - delay doubles with each retry attempt
    Exponential,
    /// Exponential backoff with full jitter - random delay between 0 and exponential value
    ExponentialWithJitter,
}

/// Statistics about retry attempts
#[derive(Debug, Clone, Default)]
pub struct RetryStats {
    /// Number of retry attempts made
    pub attempts: u64,
    /// Total time spent in retry delays (milliseconds)
    pub total_delay_ms: u64,
    /// Whether the operation eventually succeeded
    pub succeeded: bool,
    /// The last error encountered (if operation failed)
    pub last_error: Option<String>,
    /// Timestamps of each retry attempt
    pub retry_timestamps: Vec<std::time::SystemTime>,
}

impl RetryStats {
    /// Create a new RetryStats instance
    pub fn new() -> Self {
        Self {
            retry_timestamps: Vec::new(),
            ..Default::default()
        }
    }

    /// Record a retry attempt
    pub fn record_attempt(&mut self, delay_ms: u64) {
        self.attempts += 1;
        self.total_delay_ms += delay_ms;
        self.retry_timestamps.push(std::time::SystemTime::now());
    }

    /// Mark the operation as successful
    pub fn mark_success(&mut self) {
        self.succeeded = true;
    }

    /// Record the last error encountered
    pub fn record_error(&mut self, error: &str) {
        self.last_error = Some(error.to_string());
    }
}

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u64,
    /// Base delay between retries in milliseconds
    pub base_delay_ms: u64,
    /// Backoff strategy to use
    pub backoff_strategy: BackoffStrategy,
    /// Maximum delay between retries in milliseconds (for exponential backoff)
    pub max_delay_ms: u64,
    /// Whether to track retry statistics
    pub track_stats: bool,
    /// Status codes that should trigger a retry
    pub retryable_status_codes: Vec<u16>,
    /// Status codes that should be treated as permanent failures
    pub permanent_failure_status_codes: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 500,
            backoff_strategy: BackoffStrategy::ExponentialWithJitter,
            max_delay_ms: 30000, // 30 seconds max delay
            track_stats: false,
            retryable_status_codes: vec![408, 429, 500, 502, 503, 504], // Common transient errors
            permanent_failure_status_codes: vec![400, 401, 403, 404],   // Common permanent errors
        }
    }
}

/// Calculate delay for next retry based on retry configuration
fn calculate_retry_delay(config: &RetryConfig, attempt: u64) -> u64 {
    match config.backoff_strategy {
        BackoffStrategy::Constant => config.base_delay_ms,

        BackoffStrategy::Linear => {
            let delay = config.base_delay_ms * attempt;
            std::cmp::min(delay, config.max_delay_ms)
        }

        BackoffStrategy::Exponential => {
            let delay = config.base_delay_ms * (1 << attempt.min(30)); // Prevent overflow with min(30)
            std::cmp::min(delay, config.max_delay_ms)
        }

        BackoffStrategy::ExponentialWithJitter => {
            let max_delay = config.base_delay_ms * (1 << attempt.min(30)); // Prevent overflow
            let capped_delay = std::cmp::min(max_delay, config.max_delay_ms);

            // Generate random delay between 0 and capped_delay
            use rand::Rng;
            let mut rng = rand::thread_rng();
            rng.gen_range(0..=capped_delay)
        }
    }
}

/// Checks if a status code should trigger a retry
fn should_retry_status(config: &RetryConfig, status: u16) -> bool {
    if config.permanent_failure_status_codes.contains(&status) {
        return false;
    }

    config.retryable_status_codes.contains(&status) || (status >= 500 && status < 600)
}

/// Fetches URLs for photo assets from the iCloud API
///
/// This function makes a POST request to the webasseturls endpoint with an array of photo GUIDs
/// and returns a map of GUID to URL for each asset.
///
/// # Arguments
///
/// * `client` - A reqwest HTTP client
/// * `base_url` - The base URL for API requests
/// * `photo_guids` - A slice of photo GUIDs to fetch URLs for
///
/// # Returns
///
/// A HashMap mapping from photo GUID to its full URL
pub async fn get_asset_urls(
    client: &Client,
    base_url: &str,
    photo_guids: &[String],
) -> Result<HashMap<String, String>, ApiError> {
    get_asset_urls_with_config(client, base_url, photo_guids, RetryConfig::default()).await
}

/// Fetches URLs for photo assets from the iCloud API with custom retry configuration
///
/// This function makes a POST request to the webasseturls endpoint with an array of photo GUIDs
/// and returns a map of GUID to URL for each asset.
///
/// # Arguments
///
/// * `client` - A reqwest HTTP client
/// * `base_url` - The base URL for API requests
/// * `photo_guids` - A slice of photo GUIDs to fetch URLs for
/// * `retry_config` - Configuration for retry behavior
///
/// # Returns
///
/// A HashMap mapping from photo GUID to its full URL
pub async fn get_asset_urls_with_config(
    client: &Client,
    base_url: &str,
    photo_guids: &[String],
    retry_config: RetryConfig,
) -> Result<HashMap<String, String>, ApiError> {
    // Early exit if there are no photo GUIDs
    if photo_guids.is_empty() {
        log_warning("No photo GUIDs provided to get_asset_urls");
        return Ok(HashMap::new());
    }

    // Build the URL for the webasseturls endpoint
    let url = format!("{}webasseturls", base_url);

    // Create the payload with the photo GUIDs
    let payload = json!({ "photoGuids": photo_guids });

    // Initialize retry statistics if tracking is enabled
    let mut stats = if retry_config.track_stats {
        Some(RetryStats::new())
    } else {
        None
    };

    // Execute the HTTP request with retries
    let result = execute_with_retry(
        || async {
            // Make the POST request
            let resp = client.post(&url).json(&payload).send().await?;

            // Special case: handle 400 Bad Request differently for this endpoint
            if resp.status().as_u16() == 400 {
                log_warning("webasseturls request failed with 400 Bad Request. The API may be rejecting batch requests. Returning empty map to continue with partial functionality.");
                return Ok(HashMap::new());
            }
            // Check if the request was successful
            if !resp.status().is_success() {
                return Err(ApiError::RequestError {
                    status: Some(resp.status().as_u16()),
                    message: format!("webasseturls request failed"),
                });
            }
            // Parse the response as JSON
            let data: serde_json::Value = resp.json().await?;
            // Validate the API response against expected schema
            validate_webasseturls_response(&data)?;
            // Process the response and extract URLs
            process_webasseturls_response(&data)
        },
        &retry_config,
        stats.as_mut(),
    ).await;

    // If tracking stats, log them
    if let Some(stats) = stats {
        if stats.attempts > 0 {
            log_warning(&format!(
                "Request to {} required {} retries over {}ms{}",
                url,
                stats.attempts,
                stats.total_delay_ms,
                if stats.succeeded {
                    " and eventually succeeded"
                } else {
                    " and still failed"
                }
            ));
        }
    }

    result
}

/// Validate the API response for webasseturls endpoint
fn validate_webasseturls_response(data: &serde_json::Value) -> Result<(), ApiError> {
    // Validate the API response against expected schema
    let issues = validate_api_schema(data, "webasseturls");
    if !issues.is_empty() {
        // Log all validation issues as warnings
        for (field, failure) in &issues {
            match failure {
                ValidationFailure::Missing => {
                    log_warning(&format!("Schema validation: Missing field '{}'", field));
                }
                ValidationFailure::WrongType => {
                    log_warning(&format!(
                        "Schema validation: Field '{}' has wrong type",
                        field
                    ));
                }
                ValidationFailure::InvalidValue(msg) => {
                    log_warning(&format!(
                        "Schema validation: Field '{}' has invalid value: {}",
                        field, msg
                    ));
                }
            }
        }

        // Critical schema issues should be treated as errors
        let critical_issues = issues.iter().any(|(field, _)| field == "items");
        if critical_issues {
            return Err(ApiError::JsonParseError(format!(
                "Critical schema validation issues: {}",
                issues.len()
            )));
        }

        // Non-critical issues are logged but processing continues
        log_warning(&format!(
            "API response has {} schema validation issues",
            issues.len()
        ));
    }

    Ok(())
}

/// Process the webasseturls response to extract URLs
fn process_webasseturls_response(
    data: &serde_json::Value,
) -> Result<HashMap<String, String>, ApiError> {
    let mut results = HashMap::new();

    // Extract the items field which is required for this API
    let items = match data.get("items") {
        Some(items) => items,
        None => {
            log_warning("Missing 'items' field in webasseturls response");

            // This is a critical error - without items we can't proceed
            return Err(ApiError::MissingFieldError("items".to_string()));
        }
    };

    // Items must be an object mapping GUIDs to URL data
    let items_obj = match items.as_object() {
        Some(obj) => obj,
        None => {
            log_warning("'items' is not an object in webasseturls response");

            // This is also critical - we need the object structure
            return Err(ApiError::JsonParseError(
                "'items' field is not an object".to_string(),
            ));
        }
    };

    // Process each item in the map
    for (guid, value) in items_obj.iter() {
        // Extract URL components with strict validation
        // url_location is required
        let url_location = match value.get("url_location") {
            Some(loc) => match loc.as_str() {
                Some(s) if !s.is_empty() => s,
                Some(_) => {
                    log_warning(&format!("Empty url_location for guid {}", guid));
                    continue;
                }
                None => {
                    log_warning(&format!("url_location is not a string for guid {}", guid));
                    continue;
                }
            },
            None => {
                log_warning(&format!("Missing url_location for guid {}", guid));
                continue;
            }
        };

        // url_path is required
        let url_path = match value.get("url_path") {
            Some(path) => match path.as_str() {
                Some(s) if !s.is_empty() => s,
                Some(_) => {
                    log_warning(&format!("Empty url_path for guid {}", guid));
                    continue;
                }
                None => {
                    log_warning(&format!("url_path is not a string for guid {}", guid));
                    continue;
                }
            },
            None => {
                log_warning(&format!("Missing url_path for guid {}", guid));
                continue;
            }
        };

        // Build the full URL and add to results
        let full_url = format!("https://{}{}", url_location, url_path);
        results.insert(guid.to_string(), full_url);
    }

    Ok(results)
}

/// Executes an async operation with retry logic based on configuration
///
/// # Arguments
///
/// * `operation` - Async operation to execute (as a closure)
/// * `config` - Retry configuration
/// * `stats` - Optional statistics to track (mutated if provided)
///
/// # Returns
///
/// Result of the operation
async fn execute_with_retry<F, Fut, T>(
    operation: F,
    config: &RetryConfig,
    mut stats: Option<&mut RetryStats>,
) -> Result<T, ApiError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, ApiError>>,
{
    let mut attempt: u64 = 0;
    let mut last_error = None;

    loop {
        // Check if we've exceeded max retries
        if attempt >= config.max_retries {
            break;
        }

        // Only sleep before retries (not before first attempt)
        if attempt > 0 {
            // Calculate delay for this retry attempt
            let delay_ms = calculate_retry_delay(config, attempt);

            // Record the attempt if tracking stats
            if let Some(stats_ref) = stats.as_mut() {
                stats_ref.record_attempt(delay_ms);
            }

            // Sleep before retry
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        }

        // Execute the operation
        match operation().await {
            Ok(result) => {
                // Record success if tracking stats
                if let Some(stats_ref) = stats.as_mut() {
                    stats_ref.mark_success();
                }

                return Ok(result);
            }
            Err(err) => {
                // Determine if we should retry based on the error
                let should_retry = match &err {
                    ApiError::NetworkError(_) => true, // Network errors are generally transient
                    ApiError::RequestError { status, .. } => {
                        if let Some(status_code) = status {
                            should_retry_status(config, *status_code)
                        } else {
                            true // If no status code available, retry by default
                        }
                    }
                    ApiError::JsonParseError(_) => false, // JSON parse errors are unlikely to be resolved by retry
                    ApiError::MissingFieldError(_) => false, // Missing fields won't appear on retry
                    _ => true,                            // Default to retry for other error types
                };

                if should_retry {
                    // Save the error and increment attempt counter
                    if let Some(stats_ref) = stats.as_mut() {
                        stats_ref.record_error(&err.to_string());
                    }

                    last_error = Some(err);
                    attempt += 1;

                    continue; // Try again
                } else {
                    // Don't retry this type of error
                    return Err(err);
                }
            }
        }
    }

    // If we get here, all retries failed
    Err(last_error
        .unwrap_or_else(|| ApiError::RetryError("Operation failed after retries".to_string())))
}
