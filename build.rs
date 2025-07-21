use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=models/swerank-embed-small.safetensors");
    
    // Check if we're building a release
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    
    // Check if the model file exists
    let model_path = Path::new("models/swerank-embed-small.safetensors");
    
    if model_path.exists() {
        println!("cargo:rustc-cfg=has_bundled_model");
        
        // For release builds, we'll embed the model
        if profile == "release" {
            println!("cargo:rustc-cfg=embed_model");
        }
    } else {
        println!("cargo:warning=Model file not found: models/swerank-embed-small.safetensors");
        println!("cargo:warning=Run ./scripts/download-models.sh to download the model");
        println!("cargo:warning=Semantic search will use fallback mode");
    }
}