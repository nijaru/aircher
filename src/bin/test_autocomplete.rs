use aircher::ui::autocomplete::AutocompleteEngine;

#[derive(Debug)]
struct TestCase {
    input: &'static str,
    cursor_pos: usize,
    expected_contains: Vec<&'static str>,
    expected_excludes: Vec<&'static str>,
    description: &'static str,
}

#[derive(Debug)]
struct TestResult {
    passed: bool,
    #[allow(dead_code)] // Used for debugging
    input: String,
    found: Vec<String>,
    missing: Vec<String>,
    unexpected: Vec<String>,
}

fn run_test_case(engine: &mut AutocompleteEngine, test: &TestCase) -> TestResult {
    engine.generate_suggestions(test.input, test.cursor_pos).unwrap();
    
    let found_completions: Vec<String> = engine.suggestions.iter()
        .map(|s| s.completion.clone())
        .collect();
    
    let mut missing = Vec::new();
    let mut unexpected = Vec::new();
    
    // Check for expected items
    for expected in &test.expected_contains {
        if !found_completions.contains(&expected.to_string()) {
            missing.push(expected.to_string());
        }
    }
    
    // Check for items that should be excluded
    for excluded in &test.expected_excludes {
        if found_completions.contains(&excluded.to_string()) {
            unexpected.push(excluded.to_string());
        }
    }
    
    TestResult {
        passed: missing.is_empty() && unexpected.is_empty(),
        input: test.input.to_string(),
        found: found_completions,
        missing,
        unexpected,
    }
}

fn print_detailed_results(engine: &mut AutocompleteEngine, input: &str, cursor_pos: usize) {
    engine.generate_suggestions(input, cursor_pos).unwrap();
    
    if engine.suggestions.is_empty() {
        println!("    No suggestions");
        return;
    }
    
    println!("    {} suggestions found:", engine.suggestions.len());
    for (i, sug) in engine.suggestions.iter().enumerate() {
        println!("      {}. {} (conf: {:.2}) - {}", 
                i + 1, sug.completion, sug.confidence, sug.description);
    }
}

fn simulate_tui_typing(engine: &mut AutocompleteEngine, input: &str) -> Vec<String> {
    // Simulate TUI behavior: type character by character
    let mut current_input = String::new();
    
    for ch in input.chars() {
        current_input.push(ch);
        let cursor_pos = current_input.len();
        
        // This is what the TUI does on each character
        let _ = engine.generate_suggestions(&current_input, cursor_pos);
        
        println!("  After typing '{}': {:?}", current_input, 
                engine.suggestions.iter().map(|s| &s.completion).collect::<Vec<_>>());
    }
    
    // Return final suggestions
    engine.suggestions.iter().map(|s| s.completion.clone()).collect()
}

fn main() {
    let mut engine = AutocompleteEngine::new();
    
    println!("Autocomplete Engine Test Suite");
    println!("==============================");
    
    // Test TUI-style incremental typing
    println!("\n=== TUI SIMULATION TEST ===");
    println!("Simulating how TUI types character by character:");
    
    println!("\nSimulating typing '/mo':");
    let mo_results = simulate_tui_typing(&mut engine, "/mo");
    println!("Final /mo results: {:?}", mo_results);
    
    println!("\nSimulating typing '/co':");
    let co_results = simulate_tui_typing(&mut engine, "/co");
    println!("Final /co results: {:?}", co_results);
    
    // Comprehensive test cases covering reported issues
    let test_cases = vec![
        TestCase {
            input: "/m",
            cursor_pos: 2,
            expected_contains: vec!["/model"],
            expected_excludes: vec![],
            description: "Single letter should match /model",
        },
        TestCase {
            input: "/mo", 
            cursor_pos: 3,
            expected_contains: vec!["/model"],
            expected_excludes: vec![],
            description: "Prefix /mo should match /model",
        },
        TestCase {
            input: "/c",
            cursor_pos: 2,
            expected_contains: vec!["/clear", "/config", "/compact"],
            expected_excludes: vec![],
            description: "Single /c should match commands starting with 'c'",
        },
        TestCase {
            input: "/cl",
            cursor_pos: 3,
            expected_contains: vec!["/clear"],
            expected_excludes: vec!["/config", "/compact"],
            description: "Prefix /cl should only match /clear",
        },
        TestCase {
            input: "/co",
            cursor_pos: 3,
            expected_contains: vec!["/config", "/compact"],
            expected_excludes: vec!["/clear"],
            description: "Prefix /co should match both /config and /compact",
        },
        TestCase {
            input: "/comp",
            cursor_pos: 5,
            expected_contains: vec!["/compact"],
            expected_excludes: vec!["/config"],
            description: "Longer prefix /comp should only match /compact",
        },
        TestCase {
            input: "/conf",
            cursor_pos: 5,
            expected_contains: vec!["/config"],
            expected_excludes: vec!["/compact"],
            description: "Longer prefix /conf should only match /config",
        },
    ];
    
    // Run all test cases
    let mut passed = 0;
    let mut failed = 0;
    
    for test in &test_cases {
        println!("\nTest: {}", test.description);
        println!("Input: '{}' (cursor at {})", test.input, test.cursor_pos);
        
        let result = run_test_case(&mut engine, test);
        
        if result.passed {
            println!("✅ PASSED");
            passed += 1;
        } else {
            println!("❌ FAILED");
            failed += 1;
            
            if !result.missing.is_empty() {
                println!("   Missing expected: {:?}", result.missing);
            }
            if !result.unexpected.is_empty() {
                println!("   Unexpected found: {:?}", result.unexpected);
            }
        }
        
        println!("   Found: {:?}", result.found);
    }
    
    // Detailed analysis of problem cases
    println!("\n{}", "=".repeat(50));
    println!("DETAILED ANALYSIS OF PROBLEM CASES");
    println!("{}", "=".repeat(50));
    
    let problem_cases = vec![
        ("/mo", 3, "User reported: /mo doesn't show /model"),
        ("/co", 3, "User reported: /co only shows /config, not /compact"),
    ];
    
    for (input, cursor, description) in problem_cases {
        println!("\n{}", description);
        println!("Testing: '{}'", input);
        print_detailed_results(&mut engine, input, cursor);
    }
    
    // Edge case analysis
    println!("\n{}", "=".repeat(50));
    println!("EDGE CASE ANALYSIS");
    println!("{}", "=".repeat(50));
    
    let edge_cases = vec![
        ("/", 1, "Just slash - should show all commands"),
        ("/model", 6, "Full command - should show exact match"),
        ("/s", 2, "Should match /search and /sessions"),
        ("/h", 2, "Should match /help"),
    ];
    
    for (input, cursor, description) in edge_cases {
        println!("\n{}", description);
        println!("Testing: '{}'", input);
        print_detailed_results(&mut engine, input, cursor);
    }
    
    // Summary
    println!("\n{}", "=".repeat(50));
    println!("TEST SUMMARY");
    println!("{}", "=".repeat(50));
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Total:  {}", passed + failed);
    
    if failed > 0 {
        println!("\n⚠️  Some tests failed. The autocomplete behavior may not match user expectations.");
        std::process::exit(1);
    } else {
        println!("\n✅ All tests passed!");
    }
}