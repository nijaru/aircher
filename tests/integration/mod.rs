pub mod providers;
pub mod error_handling;

// Integration test module
// 
// This module contains integration tests that verify the complete functionality
// of the Aircher system with real external services.
//
// Tests in this module:
// - providers: Require actual API keys to be set and make real HTTP requests
// - error_handling: Test error scenarios and provider capabilities without API calls
//
// To run integration tests:
// cargo test --package aircher --test integration_tests
//
// To run tests that require API keys:
// cargo test --package aircher --test integration_tests -- --ignored