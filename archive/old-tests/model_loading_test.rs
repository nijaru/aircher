use std::path::Path;

#[test]
fn test_model_file_exists() {
    // Test that the model file exists
    let model_path = Path::new("models/swerank-embed-small.bin");
    assert!(model_path.exists(), "Model file should exist at: {}", model_path.display());
}

#[test]
fn test_include_bytes_works() {
    // Test that include_bytes! macro works
    let model_data = include_bytes!("../models/swerank-embed-small.bin");
    assert!(!model_data.is_empty(), "Model data should not be empty");
    println!("Model data size: {} bytes", model_data.len());
}

#[test]
fn test_model_data_content() {
    // Test that we can read the actual model data
    let model_data = include_bytes!("../models/swerank-embed-small.bin");
    println!("First 20 bytes: {:?}", &model_data[..std::cmp::min(20, model_data.len())]);
}
