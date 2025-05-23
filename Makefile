.PHONY: build test clean install lint fmt vet deps docker release help

# Variables
BINARY_NAME=aircher
MAIN_PACKAGE=./cmd/aircher
BUILD_DIR=build
VERSION?=dev
COMMIT?=$(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BUILD_DATE?=$(shell date -u +"%Y-%m-%dT%H:%M:%SZ")
LDFLAGS=-ldflags "-X main.version=$(VERSION) -X main.commit=$(COMMIT) -X main.date=$(BUILD_DATE)"

# Go parameters
GOCMD=go
GOBUILD=$(GOCMD) build
GOCLEAN=$(GOCMD) clean
GOTEST=$(GOCMD) test
GOGET=$(GOCMD) get
GOMOD=$(GOCMD) mod
GOFMT=gofmt
GOVET=$(GOCMD) vet

# Default target
all: clean deps fmt vet lint test build

# Build the binary
build:
	@echo "Building $(BINARY_NAME)..."
	@mkdir -p $(BUILD_DIR)
	$(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/$(BINARY_NAME) $(MAIN_PACKAGE)

# Install the binary to $GOPATH/bin or $GOBIN
install:
	@echo "Installing $(BINARY_NAME)..."
	$(GOBUILD) $(LDFLAGS) -o $(GOPATH)/bin/$(BINARY_NAME) $(MAIN_PACKAGE)

# Run tests
test:
	@echo "Running tests..."
	$(GOTEST) -v -race -coverprofile=coverage.out ./...

# Run tests with coverage
test-coverage: test
	@echo "Generating coverage report..."
	$(GOCMD) tool cover -html=coverage.out -o coverage.html
	@echo "Coverage report generated: coverage.html"

# Clean build artifacts
clean:
	@echo "Cleaning..."
	$(GOCLEAN)
	rm -rf $(BUILD_DIR)
	rm -f coverage.out coverage.html

# Download dependencies
deps:
	@echo "Downloading dependencies..."
	$(GOMOD) download
	$(GOMOD) tidy

# Format code (uses gofumpt for better formatting)
fmt:
	@echo "Formatting code..."
	@which gofumpt > /dev/null || (echo "gofumpt not found. Install with: go install mvdan.cc/gofumpt@latest" && exit 1)
	gofumpt -w .

# Format code (fallback to standard gofmt)
fmt-std:
	@echo "Formatting code with standard gofmt..."
	$(GOFMT) -s -w .

# Vet code
vet:
	@echo "Vetting code..."
	$(GOVET) ./...

# Lint code (requires golangci-lint)
lint:
	@echo "Linting code..."
	@which golangci-lint > /dev/null || (echo "golangci-lint not found. Install with: go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest" && exit 1)
	golangci-lint run

# Run development version
dev: build
	@echo "Running development version..."
	./$(BUILD_DIR)/$(BINARY_NAME)

# Build for multiple platforms
release:
	@echo "Building release binaries..."
	@mkdir -p $(BUILD_DIR)/release
	
	# Linux amd64
	GOOS=linux GOARCH=amd64 $(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/release/$(BINARY_NAME)-linux-amd64 $(MAIN_PACKAGE)
	
	# Linux arm64
	GOOS=linux GOARCH=arm64 $(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/release/$(BINARY_NAME)-linux-arm64 $(MAIN_PACKAGE)
	
	# macOS amd64
	GOOS=darwin GOARCH=amd64 $(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/release/$(BINARY_NAME)-darwin-amd64 $(MAIN_PACKAGE)
	
	# macOS arm64
	GOOS=darwin GOARCH=arm64 $(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/release/$(BINARY_NAME)-darwin-arm64 $(MAIN_PACKAGE)
	
	# Windows amd64
	GOOS=windows GOARCH=amd64 $(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/release/$(BINARY_NAME)-windows-amd64.exe $(MAIN_PACKAGE)
	
	@echo "Release binaries built in $(BUILD_DIR)/release/"

# Build Docker image
docker:
	@echo "Building Docker image..."
	docker build -t $(BINARY_NAME):$(VERSION) .
	docker tag $(BINARY_NAME):$(VERSION) $(BINARY_NAME):latest

# Run quality checks
check: fmt vet lint test

# Initialize development environment
init:
	@echo "Initializing development environment..."
	$(GOMOD) download
	@echo "Installing development tools..."
	go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
	go install mvdan.cc/gofumpt@latest
	go install honnef.co/go/tools/cmd/staticcheck@latest
	@echo "Development environment ready!"

# Install all development tools
tools:
	@echo "Installing development tools..."
	go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
	go install mvdan.cc/gofumpt@latest
	go install honnef.co/go/tools/cmd/staticcheck@latest

# Update all development tools
tools-update:
	@echo "Updating development tools..."
	go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
	go install mvdan.cc/gofumpt@latest
	go install honnef.co/go/tools/cmd/staticcheck@latest

# Show help
help:
	@echo "Available targets:"
	@echo "  build         - Build the binary"
	@echo "  test          - Run tests"
	@echo "  test-coverage - Run tests with coverage report"
	@echo "  clean         - Clean build artifacts"
	@echo "  deps          - Download dependencies"
	@echo "  fmt           - Format code with gofumpt"
	@echo "  fmt-std       - Format code with standard gofmt"
	@echo "  vet           - Vet code"
	@echo "  lint          - Lint code (requires golangci-lint)"
	@echo "  install       - Install binary to GOPATH/bin"
	@echo "  dev           - Build and run development version"
	@echo "  release       - Build for multiple platforms"
	@echo "  docker        - Build Docker image"
	@echo "  check         - Run all quality checks"
	@echo "  init          - Initialize development environment"
	@echo "  tools         - Install all development tools"
	@echo "  tools-update  - Update all development tools"
	@echo "  help          - Show this help"