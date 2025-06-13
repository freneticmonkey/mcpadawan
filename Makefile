APP_NAME := mcpadawan

.PHONY: all run build check fmt clean driver-check

all: run

## Run the MCP server
run:
	cargo run

## Build the release binary
build:
	cargo build --release

## Check for compile errors and warnings
check:
	cargo check

## Format code using rustfmt
fmt:
	cargo fmt

## Clean build artifacts
clean:
	cargo clean

## Check if ViGEmBus is installed (Windows only)
driver-check:
	@echo Checking for ViGEmBus driver...
	@powershell -Command "Get-PnpDevice -FriendlyName '*ViGEm*' | Format-Table -AutoSize" || echo "⚠️ ViGEmBus driver not found. Please install from https://vigem.org"