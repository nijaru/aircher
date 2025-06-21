package main

import (
	"fmt"
	"os"

	"github.com/aircher/aircher/internal/core"
	"github.com/spf13/cobra"
)

var (
	version = "dev"
	commit  = "unknown"
	date    = "unknown"
)

var (
	prompt       string
	continue_    bool
	resume       string
	outputFormat string
	provider     string
)

func main() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

var rootCmd = &cobra.Command{
	Use:   "aircher",
	Short: "Next-generation AI coding assistant with multi-provider support",
	Long: `Aircher is a command-line AI coding assistant that works with any LLM provider
while providing superior context management, autonomous web search, and intelligent automation.

Examples:
  aircher                              # Start interactive REPL
  aircher "explain this project"       # REPL with initial prompt
  aircher -c                          # Continue last conversation
  aircher -r "session-id"             # Resume specific session
  aircher -p "query"                  # One-shot query, then exit
  cat file.go | aircher -p "review"   # Process piped content`,
	RunE: func(cmd *cobra.Command, args []string) error {
		// Initialize Aircher core
		aircher, err := core.NewAircher()
		if err != nil {
			return fmt.Errorf("failed to initialize Aircher: %w", err)
		}
		defer aircher.Close()

		// Handle different execution modes
		if prompt != "" {
			// Non-interactive mode with prompt
			return aircher.RunNonInteractive(prompt, outputFormat, provider)
		}

		if continue_ {
			// Continue last conversation
			return aircher.ContinueLastConversation()
		}

		if resume != "" {
			// Resume specific session
			return aircher.ResumeSession(resume)
		}

		// Check for initial prompt argument
		var initialPrompt string
		if len(args) > 0 {
			initialPrompt = args[0]
		}

		// Default: Interactive REPL mode
		return aircher.RunInteractive(initialPrompt)
	},
}

var configCmd = &cobra.Command{
	Use:   "config",
	Short: "Interactive configuration management",
	RunE: func(cmd *cobra.Command, args []string) error {
		aircher, err := core.NewAircher()
		if err != nil {
			return fmt.Errorf("failed to initialize Aircher: %w", err)
		}
		defer aircher.Close()

		return aircher.RunConfigManager()
	},
}

var initCmd = &cobra.Command{
	Use:   "init",
	Short: "Initialize project with AGENTS.md",
	RunE: func(cmd *cobra.Command, args []string) error {
		aircher, err := core.NewAircher()
		if err != nil {
			return fmt.Errorf("failed to initialize Aircher: %w", err)
		}
		defer aircher.Close()

		return aircher.InitializeProject()
	},
}

var doctorCmd = &cobra.Command{
	Use:   "doctor",
	Short: "Health diagnostics and troubleshooting",
	RunE: func(cmd *cobra.Command, args []string) error {
		aircher, err := core.NewAircher()
		if err != nil {
			return fmt.Errorf("failed to initialize Aircher: %w", err)
		}
		defer aircher.Close()

		return aircher.RunHealthCheck()
	},
}

var updateCmd = &cobra.Command{
	Use:   "update",
	Short: "Self-update with rollback support",
	RunE: func(cmd *cobra.Command, args []string) error {
		aircher, err := core.NewAircher()
		if err != nil {
			return fmt.Errorf("failed to initialize Aircher: %w", err)
		}
		defer aircher.Close()

		return aircher.RunSelfUpdate()
	},
}

var versionCmd = &cobra.Command{
	Use:   "version",
	Short: "Print version information",
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Printf("Aircher %s\n", version)
		fmt.Printf("  commit: %s\n", commit)
		fmt.Printf("  built:  %s\n", date)
	},
}

func init() {
	// Root command flags
	rootCmd.Flags().StringVarP(&prompt, "prompt", "p", "", "One-shot query, then exit")
	rootCmd.Flags().BoolVarP(&continue_, "continue", "c", false, "Continue last conversation")
	rootCmd.Flags().StringVarP(&resume, "resume", "r", "", "Resume specific session")
	rootCmd.Flags().StringVar(&outputFormat, "output-format", "text", "Output format: text, json, markdown")
	rootCmd.Flags().StringVar(&provider, "provider", "", "LLM provider: openai, claude, gemini, ollama")

	// Add subcommands
	rootCmd.AddCommand(configCmd)
	rootCmd.AddCommand(initCmd)
	rootCmd.AddCommand(doctorCmd)
	rootCmd.AddCommand(updateCmd)
	rootCmd.AddCommand(versionCmd)
}
