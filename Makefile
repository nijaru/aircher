.PHONY: build test clean lint fmt release help validate-tasks new-task progress

# Rust build commands
build:
	cargo build

build-release:
	cargo build --release

test:
	cargo test --all

lint:
	cargo clippy --all-targets --all-features

fmt:
	cargo fmt

check: fmt lint test

# Task management utilities
validate-tasks:
	@echo "Validating tasks.json structure..."
	@jq empty docs/tasks/tasks.json && echo "✅ Valid JSON" || (echo "❌ Invalid JSON" && exit 1)

new-task:
	@read -p "Task ID: " id; \
	read -p "Title: " title; \
	read -p "Priority (critical/high/medium/low): " priority; \
	read -p "Description: " description; \
	jq --arg id "$$id" --arg title "$$title" --arg priority "$$priority" --arg description "$$description" \
	'.tasks[$$id] = {"title": $$title, "status": "pending", "priority": $$priority, "description": $$description, "acceptance_criteria": []}' \
	docs/tasks/tasks.json > tmp.json && mv tmp.json docs/tasks/tasks.json && \
	echo "✅ Task $$id created"

progress:
	@echo "📊 Current Task Status:"
	@jq -r '.tasks | to_entries | group_by(.value.status) | map("\(.[0].value.status): \(length) tasks") | .[]' docs/tasks/tasks.json
	@echo ""
	@echo "🎯 Next Priority Tasks:"
	@jq -r '.priorities.next_sequence[] as $$id | .tasks[$$id] | select(.status != "completed") | "- \(.title) (\(.status))"' docs/tasks/tasks.json

current-tasks:
	@echo "📋 Tasks In Progress:"
	@jq -r '.tasks | to_entries | map(select(.value.status == "in_progress")) | .[] | "- \(.key): \(.value.title)"' docs/tasks/tasks.json
	@echo ""
	@echo "⏳ Pending High Priority:"
	@jq -r '.tasks | to_entries | map(select(.value.status == "pending" and (.value.priority == "critical" or .value.priority == "high"))) | .[] | "- \(.key): \(.value.title)"' docs/tasks/tasks.json

# Git hooks setup
setup-hooks:
	@echo "Setting up git hooks..."
	@echo '#!/bin/bash\n# Validate tasks.json before commit\nif [ -f docs/tasks/tasks.json ]; then\n  jq empty docs/tasks/tasks.json || {\n    echo "Error: Invalid JSON in docs/tasks/tasks.json"\n    exit 1\n  }\nfi' > .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "✅ Pre-commit hook installed"

# Development setup
dev-setup: setup-hooks
	@echo "Setting up development environment..."
	@rustup update stable
	@echo "✅ Development environment ready"

help:
	@echo "🛠️  Aircher Development Commands:"
	@echo ""
	@echo "Build & Test:"
	@echo "  build         - Build debug version"
	@echo "  build-release - Build optimized release version"
	@echo "  test          - Run all tests"
	@echo "  lint          - Run clippy linting"
	@echo "  fmt           - Format code"
	@echo "  check         - Run fmt + lint + test"
	@echo ""
	@echo "Task Management:"
	@echo "  validate-tasks - Check tasks.json is valid JSON"
	@echo "  new-task      - Create new task interactively"
	@echo "  progress      - Show overall progress summary"
	@echo "  current-tasks - Show active and priority tasks"
	@echo ""
	@echo "Development:"
	@echo "  setup-hooks   - Install git pre-commit hooks"
	@echo "  dev-setup     - Complete development environment setup"
	@echo "  help          - Show this help"