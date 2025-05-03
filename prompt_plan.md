Below is a comprehensive plan followed by a set of iterative “micro-roadmaps,” and finally a series of TDD-style prompts for a code-generation LLM. Each prompt is in its own code block. The overall structure is:

1. **High-Level Blueprint**
2. **Breakdown into Iterative Chunks**
3. **Further Subdivision of Each Chunk**
4. **Series of LLM Prompts**

---

## 1. High-Level Blueprint

1. **Project Setup**

   * Create a new Rust library project (`cargo new icloud-album-parser --lib`).
   * Configure `Cargo.toml` dependencies: `reqwest`, `serde`, `serde_json`, `tokio` (if asynchronous), etc.
   * Set up a test framework (built-in Rust test harness).

2. **Data Model Definitions**

   * Define data structures mirroring the TypeScript interface:

     * `Derivative`, `Image`, `Metadata`, `ApiResponse`, and possibly a top-level `ICloudResponse`.
   * Ensure they support JSON deserialization (via `serde`).

3. **Token Parsing / Base URL Generation**

   * Implement logic that replicates the TypeScript base62 parse (if needed).
   * Provide tests to confirm correctness (e.g., we feed in known tokens → confirm expected base URL).

4. **Redirect Handling**

   * Implement logic to detect a 330 “redirect” and build a new base URL accordingly.
   * Write tests with mocked HTTP responses returning status=330 and checking if the new URL is created properly.

5. **Fetching Shared Album Metadata**

   * Write a function that hits the `webstream` endpoint and returns a typed struct of metadata + photos.
   * Test using a mock or stub endpoint to validate successful parsing of typical Apple JSON responses.

6. **Fetching Asset URLs**

   * Write a function that calls `webasseturls` with a list of photo GUIDs, returns a map of GUID → URL.
   * Test with mock/stub data.

7. **Enriching Photos with URLs**

   * Combine the photo objects from “webstream” and the URL mapping from “webasseturls”.
   * Return a final result with all derivatives populated with their actual URLs.
   * Write tests checking that each photo’s derivative is assigned the correct URL.

8. **Public `get_icloud_photos` Entry Point**

   * Integrate all steps above.
   * Possibly accept a “token” → produce final typed response.

9. **Documentation & Cleanup**

   * Provide a comprehensive README describing how to use the library.
   * Final pass for testing, edge cases, etc.

---

## 2. Breakdown Into Iterative Chunks

Below is a more incremental breakdown—each chunk can be done in sequence:

1. **Initialize Cargo Project**

   * Create the project.
   * Set up dependencies in `Cargo.toml`.
   * Push code to a repository, ensure CI setup if desired.

2. **Create Basic Data Models**

   * `Derivative`, `Image`, `Metadata`, `ApiResponse`, etc.
   * Mark with `#[derive(Serialize, Deserialize, Debug)]`.
   * Add minimal tests to confirm they can be serialized/deserialized from sample JSON.

3. **Implement Base URL Logic**

   * Write a function `get_base_url(token: &str) -> String`.
   * Replicate or approximate the base62 approach from the TS code.
   * Test with known tokens → known outputs.

4. **Implement Redirect Handling**

   * `get_redirected_base_url(client, base_url, token) -> String`.
   * Use a mocked server or a test library to confirm handling of 330.

5. **Fetch `webstream` (metadata/photos)**

   * `get_api_response(client, base_url) -> (Vec<Image>, Metadata)`.
   * Test with sample JSON responses.

6. **Fetch `webasseturls`**

   * `get_asset_urls(client, base_url, &photo_guids) -> HashMap<String, String>`.
   * Test with sample JSON responses.

7. **Enrich Photos**

   * `enrich_photos_with_urls(photos, all_urls)`.
   * Confirm correct assignment in tests.

8. **Public Entry**

   * `get_icloud_photos(token: &str) -> ICloudResponse`.
   * Calls all substeps in order.
   * Add an integration test with a fully mocked flow.

9. **Documentation & Cleanup**

   * README usage instructions
   * Possibly add an example in `/examples`
   * Confirm project structure is clean.

---

## 3. Further Subdivision of Each Chunk

To ensure each step is “just right” in size, we can break them down a bit more:

### Chunk 1: Initialize Cargo Project

1. `cargo new icloud-album-parser --lib`
2. Add `reqwest`, `serde`, `serde_json`, `tokio` to `[dependencies]`.
3. Confirm it compiles with `cargo build` and runs tests with `cargo test` (should be empty but passing).

### Chunk 2: Create Basic Data Models

1. Create a `models.rs` (or similarly named) with `#[derive(Serialize, Deserialize, Debug)]` for all needed structs.
2. Write a small test in `tests/models_test.rs` that tries to deserialize from a sample snippet of JSON.

### Chunk 3: Implement Base URL Logic

1. Create a `base_url.rs` with the `get_base_url` function.
2. For tests, supply a known token (`"B0z5qAGN1JIFd3y"`) and compare results against expected.

### Chunk 4: Implement Redirect Handling

1. Create `redirect.rs` with `get_redirected_base_url`.
2. Test with a local test server or `mockito`/`httptest` crate, returning 330 with JSON body containing `"X-Apple-MMe-Host": "pXX-sharedstreams.icloud.com"`.

### Chunk 5: Fetch `webstream` (metadata/photos)

1. Create `api.rs` with `get_api_response`.
2. Use a test that hits a mock endpoint returning realistic JSON (like the example from the TS code).

### Chunk 6: Fetch `webasseturls`

1. Extend `api.rs` or new `assets.rs` to add `get_asset_urls`.
2. Test with a mock JSON response that returns a `data.items` object.

### Chunk 7: Enrich Photos

1. `enrich_photos_with_urls(photos, &urls_map)`.
2. Confirm each derivative gets a `.url` if the `checksum` is found in the map.

### Chunk 8: Public Entry

1. `get_icloud_photos(token)`.
2. Compose all prior steps in a pipeline.
3. Full integration test that mocks the entire flow (redirect → webstream → asseturls).

### Chunk 9: Documentation & Cleanup

1. `README.md` explaining usage, sample code snippet, etc.
2. Possibly small example in `examples/` folder.

---

## 4. Series of LLM Prompts

Below is a set of prompts that you (or anyone) can feed an LLM so it can build the project in small, test-driven slices. Each prompt is separated by markdown fences. You would paste them into a code-generation LLM in sequence, letting it generate code for each step.

---

### Prompt 1: Initialize Project ✅

```text
You are building a Rust library named `icloud-album-parser`. First, create the project structure with `cargo new icloud-album-parser --lib`. In the resulting `Cargo.toml`, add these dependencies:

[dependencies]
reqwest = "0.11"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["macros"] }

Explain briefly in the generated code comments what each dependency is for. Provide a minimal `lib.rs` that compiles and a stub test in `tests/` folder. We’ll keep it super basic. 
```

---

### Prompt 2: Create Basic Data Models ✅

```text
Next, create a new file `src/models.rs` to hold these data structs:

1. Derivative
2. Image
3. Metadata
4. ApiResponse
5. ICloudResponse

Derive `Serialize` and `Deserialize` from Serde. Use `Debug` as well. Use placeholders for fields if unsure about the final shape. Then create a test file `tests/models_test.rs` with a small test that constructs or deserializes a sample JSON string into these models, checking that fields parse. 
```

---

### Prompt 3: Implement Base URL Logic ✅

```text
Now create `src/base_url.rs` with a public function `get_base_url(token: &str) -> String`. 
Use base62 logic or a simplified approach, referencing code from the TypeScript version if needed. 
Add a test in `tests/base_url_test.rs` that checks a known token and the expected output. 
Make sure the code compiles and tests pass.
```

---

### Prompt 4: Implement Redirect Handling ✅

```text
Add a new file `src/redirect.rs` with the function:
```

pub async fn get\_redirected\_base\_url(
client: \&reqwest::Client,
base\_url: \&str,
token: \&str
) -> String

```
It should POST to `base_url + "webstream"` with a JSON body `{ "streamCtag": null }`.
If the response status is 330, parse the JSON for `"X-Apple-MMe-Host"` and build a new URL like:
`"https://HOST/TOKEN/sharedstreams/"`. Otherwise, return `base_url`.
Write a test in `tests/redirect_test.rs` that uses `mockito` or similar to simulate a 330 response with the expected JSON. 
```

---

### Prompt 5: Fetch `webstream` (metadata/photos) ✅

```text
In `src/api.rs`, create a function:
```

pub async fn get\_api\_response(
client: \&reqwest::Client,
base\_url: \&str
) -> Result<(Vec<Image>, Metadata), Box<dyn std::error::Error>>

```
It should POST to `base_url + "webstream"` with `{ "streamCtag": null }`. Parse the result into `Vec<Image>` and `Metadata`. Use the data structures from `models.rs`. Mock the response in `tests/api_test.rs` with a realistic JSON. Confirm the function returns the expected data. 
```

---

### Prompt 6: Fetch `webasseturls` ✅

```text
Still in `src/api.rs` (or a new file `src/assets.rs`), add:
```

pub async fn get\_asset\_urls(
client: \&reqwest::Client,
base\_url: \&str,
photo\_guids: &\[String]
) -> Result\<HashMap\<String, String>, Box<dyn std::error::Error>>

```
It posts to `base_url + "webasseturls"` with JSON `{"photoGuids": photo_guids}`. 
For the JSON response, parse `data.items`, where each key is a guid, and the object has `url_location` and `url_path`. Return a map of `guid -> "https://url_location + url_path"`. 
Test it in `tests/api_test.rs` or `tests/assets_test.rs` with a sample JSON. 
```

---

### Prompt 7: Enrich Photos

```text
Create a function in `src/enrich.rs`:
```

pub fn enrich\_photos\_with\_urls(
photos: \&mut \[Image],
all\_urls: \&HashMap\<String, String>
)

```
For each `Derivative` in each `Image`, if `checksum` is in `all_urls`, set the derivative’s `url` to `all_urls[checksum]`. Add a test in `tests/enrich_test.rs` that verifies the correct assignment. 
```

---

### Prompt 8: Public Entry Point

```text
Finally, create a function in `src/lib.rs` named `get_icloud_photos(token: &str) -> Result<ICloudResponse, Box<dyn std::error::Error>>` that:

1. Creates a reqwest `Client`.
2. Calls `get_base_url(token)`.
3. Calls `get_redirected_base_url(...)`.
4. Calls `get_api_response(...)` -> `(photos, metadata)`.
5. Extracts all `photo_guids` from `photos`.
6. Calls `get_asset_urls(...)`.
7. Calls `enrich_photos_with_urls(...)`.
8. Returns a final `ICloudResponse` with `metadata` and `photos`.
   
Add an integration test in `tests/integration_test.rs` or similar that mocks everything (redirect, webstream, asseturls). Ensure we get back a structured `ICloudResponse` with data assigned. 
```

---

### Prompt 9: Documentation & Cleanup

```text
Now add a README.md that documents how to use `get_icloud_photos(token)`, including a brief example. 
Ensure final code is tested, consider edge cases, and optionally create an `examples/` folder with a `main.rs` that calls `get_icloud_photos`. 
We want the final library to be ready for publication on crates.io, so ensure we have a license field and a version in Cargo.toml. 
```

---

**Conclusion**
These prompts are designed to walk a code-generation LLM through building a Rust-based library for parsing iCloud shared albums, **test-first** and step by step. Each prompt stands alone, builds on the prior code, and ensures that the final solution is integrated and testable with no “orphaned” code.

