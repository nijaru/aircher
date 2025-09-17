/// Competitive parity testing binary
///
/// This binary runs comprehensive tests to validate Aircher's competitive
/// position against Claude Code, Cursor, and GitHub Copilot.

use anyhow::Result;
use clap::{Parser, Subcommand};

use aircher::testing::{TestConfig, TestRunner};
use aircher::agent::runtime_validation::RuntimeValidator;

#[derive(Parser)]
#[command(name = "test-competitive-parity")]
#[command(about = "Run comprehensive competitive analysis tests for Aircher")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run complete test suite
    Full {
        /// Number of performance test iterations
        #[arg(short, long, default_value_t = 10)]
        iterations: u32,
        /// Run tests that require API keys
        #[arg(long)]
        with_api_tests: bool,
        /// Custom workspace path for tests
        #[arg(long, default_value = "/tmp/aircher_test_workspace")]
        workspace: String,
    },
    /// Run only runtime validation
    Runtime {
        /// Test workspace path
        #[arg(long, default_value = "/tmp/aircher_test_workspace")]
        workspace: String,
    },
    /// Run only performance benchmarks
    Performance {
        /// Number of iterations
        #[arg(short, long, default_value_t = 10)]
        iterations: u32,
    },
    /// Run only competitive parity analysis
    Competitive,
    /// Run only feature validation
    Features,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Full { iterations, with_api_tests, workspace } => {
            run_full_suite(iterations, with_api_tests, workspace).await?;
        }
        Commands::Runtime { workspace } => {
            run_runtime_validation(workspace).await?;
        }
        Commands::Performance { iterations } => {
            run_performance_tests(iterations).await?;
        }
        Commands::Competitive => {
            run_competitive_analysis().await?;
        }
        Commands::Features => {
            run_feature_validation().await?;
        }
    }

    Ok(())
}

/// Run complete test suite
async fn run_full_suite(iterations: u32, with_api_tests: bool, workspace: String) -> Result<()> {
    println!("🚀 AIRCHER COMPETITIVE PARITY ANALYSIS");
    println!("=======================================\n");

    let config = TestConfig {
        performance_iterations: iterations,
        run_api_tests: with_api_tests,
        workspace_path: workspace,
        ..Default::default()
    };

    let runner = TestRunner::new(config);
    let result = runner.run_complete_suite().await?;

    // Save results to file
    let results_json = serde_json::to_string_pretty(&result)?;
    tokio::fs::write("competitive_analysis_results.json", results_json).await?;

    println!("\n📄 Results saved to: competitive_analysis_results.json");

    // Return appropriate exit code based on results
    if result.success_rate >= 95.0 {
        println!("\n🎉 EXCELLENT: All systems passing with high reliability!");
        std::process::exit(0);
    } else if result.success_rate >= 80.0 {
        println!("\n⚠️  GOOD: Most systems working, some areas need attention");
        std::process::exit(1);
    } else {
        println!("\n❌ CRITICAL: Significant issues found, development needed");
        std::process::exit(2);
    }
}

/// Run runtime validation only
async fn run_runtime_validation(workspace: String) -> Result<()> {
    println!("🔧 RUNTIME VALIDATION");
    println!("=====================\n");

    let workspace_path = std::path::PathBuf::from(workspace);

    // Ensure workspace exists
    tokio::fs::create_dir_all(&workspace_path).await?;

    let validator = RuntimeValidator::new(workspace_path).await?;
    let results = validator.run_validation_suite().await?;

    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();

    if passed == total {
        println!("\n🎉 ALL RUNTIME VALIDATION TESTS PASSED");
        std::process::exit(0);
    } else {
        println!("\n❌ RUNTIME VALIDATION FAILURES DETECTED");
        std::process::exit(1);
    }
}

/// Run performance tests only
async fn run_performance_tests(iterations: u32) -> Result<()> {
    println!("⚡ PERFORMANCE BENCHMARKS");
    println!("========================\n");

    let config = TestConfig {
        performance_iterations: iterations,
        ..Default::default()
    };

    let performance_result = aircher::testing::performance_benchmarks::run_benchmarks(&config).await?;

    println!("\n📊 PERFORMANCE SUMMARY:");
    println!("  • Startup: {}ms", performance_result.startup_time_ms);
    println!("  • Memory: {}MB", performance_result.memory_usage_mb);
    println!("  • Response P50: {}ms", performance_result.response_time_p50_ms);
    println!("  • Response P95: {}ms", performance_result.response_time_p95_ms);
    println!("  • Tool Success: {:.1}%", performance_result.tool_success_rate * 100.0);
    println!("  • Overall Rank: {:.1}/4.0", performance_result.competitive_ranking.overall_rank);

    if performance_result.competitive_ranking.overall_rank <= 2.0 {
        println!("\n🏆 EXCELLENT PERFORMANCE: Top-tier competitive position");
        std::process::exit(0);
    } else {
        println!("\n⚠️  PERFORMANCE NEEDS IMPROVEMENT");
        std::process::exit(1);
    }
}

/// Run competitive analysis only
async fn run_competitive_analysis() -> Result<()> {
    println!("🏆 COMPETITIVE PARITY ANALYSIS");
    println!("==============================\n");

    let config = TestConfig::default();
    let parity_result = aircher::testing::competitive_parity::run_parity_tests(&config).await?;

    println!("📊 COMPETITIVE PARITY SCORES:");
    println!("  • vs Claude Code: {:.1}%", parity_result.parity_scores.vs_claude_code);
    println!("  • vs Cursor: {:.1}%", parity_result.parity_scores.vs_cursor);
    println!("  • vs GitHub Copilot: {:.1}%", parity_result.parity_scores.vs_github_copilot);

    let avg_parity = (parity_result.parity_scores.vs_claude_code +
        parity_result.parity_scores.vs_cursor +
        parity_result.parity_scores.vs_github_copilot) / 3.0;

    println!("\n🎯 OVERALL COMPETITIVE POSITION: {:.1}%", avg_parity);

    if avg_parity >= 90.0 {
        println!("🚀 MARKET READY: Strong competitive position achieved");
        std::process::exit(0);
    } else if avg_parity >= 80.0 {
        println!("⚡ COMPETITIVE: Good position with room for improvement");
        std::process::exit(1);
    } else {
        println!("🔧 DEVELOPMENT NEEDED: Significant gaps to address");
        std::process::exit(2);
    }
}

/// Run feature validation only
async fn run_feature_validation() -> Result<()> {
    println!("✅ FEATURE VALIDATION");
    println!("=====================\n");

    let config = TestConfig::default();
    let feature_result = aircher::testing::feature_validation::run_feature_tests(&config).await?;

    println!("📊 FEATURE VALIDATION RESULTS:");
    for (category, result) in &feature_result.feature_categories {
        println!("  • {}: {}/{} ({:.1}%)",
            category.replace("_", " ").to_uppercase(),
            result.working_features,
            result.total_features,
            result.success_rate);

        if !result.feature_details.is_empty() {
            let broken: Vec<_> = result.feature_details.iter()
                .filter(|f| matches!(f.status, aircher::testing::feature_validation::FeatureStatus::Broken))
                .collect();

            if !broken.is_empty() {
                println!("    Broken features:");
                for feature in broken {
                    println!("      - {}: {}", feature.name,
                        feature.error.as_ref().unwrap_or(&"Unknown error".to_string()));
                }
            }
        }
    }

    let overall_success = feature_result.passed_tests as f64 / feature_result.total_tests as f64 * 100.0;
    println!("\n🎯 OVERALL FEATURE SUCCESS: {:.1}%", overall_success);

    if overall_success >= 90.0 {
        println!("🎉 EXCELLENT: Most features working correctly");
        std::process::exit(0);
    } else if overall_success >= 75.0 {
        println!("⚠️  GOOD: Core features working, some gaps remain");
        std::process::exit(1);
    } else {
        println!("❌ CRITICAL: Major feature gaps detected");
        std::process::exit(2);
    }
}