APP_NAME := mtu
BUILD_DIR := build

.PHONY: build clean install deps help

build:
	@echo "Building $(APP_NAME)..."
	@mkdir -p $(BUILD_DIR)
	@go build -o $(BUILD_DIR)/$(APP_NAME) ./cmd/$(APP_NAME)

clean:
	@echo "Cleaning build artifacts..."
	@rm -rf $(BUILD_DIR)

deps:
	@echo "Installing dependencies..."
	@go mod tidy

help:
	@echo "MTU Optimizer - Build Commands"
	@echo ""
	@echo "  build    - Build the application"
	@echo "  clean    - Clean build artifacts"
	@echo "  deps     - Install/update dependencies"
	@echo "  help     - Show this help"
	@echo ""
	@echo "Usage after build: ./$(BUILD_DIR)/$(APP_NAME) [flags]"

.DEFAULT_GOAL := help