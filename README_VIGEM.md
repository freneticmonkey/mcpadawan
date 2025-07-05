# ViGEm Integration Configuration

This document describes how to configure ViGEm gamepad integration in mcpadawan.

## Overview

ViGEm (Virtual Gamepad Emulation) allows the application to create virtual Xbox 360 controllers that can be used by games and applications. This feature is primarily designed for Windows systems with the ViGEm Bus Driver installed.

## Disabling ViGEm Integration

You can disable ViGEm integration in two ways:

### 1. Command Line Flag

Use the `--disable-vigem` flag when starting the application:

```bash
mcpadawan --disable-vigem
```

### 2. Configuration File

Create or edit a `config.toml` file in the same directory as the executable:

```toml
# mcpadawan configuration file
# Set to true to disable ViGEm gamepad integration
disable_vigem = true
```

You can specify a custom configuration file path using the `-c` or `--config` flag:

```bash
mcpadawan --config /path/to/custom/config.toml
```

## Priority

The command line flag takes precedence over the configuration file. If both are specified, the CLI flag will be used.

## Compilation

### With ViGEm Support (Windows)

To compile with ViGEm support (default on Windows):

```bash
cargo build --features vigem
```

### Without ViGEm Support

To compile without ViGEm support (useful for non-Windows platforms):

```bash
cargo build
```

## Behavior

- When ViGEm is disabled, the `/gamepad` endpoint will return an error message
- When ViGEm is not compiled in, the application will inform you that the feature is missing
- All other functionality (mouse, keyboard, screenshots, process management) works independently of ViGEm

## Error Handling

The application gracefully handles ViGEm initialization failures:
- If ViGEm Bus Driver is not installed, the application will continue without gamepad support
- If ViGEm connection fails, an error message will be displayed but the application will continue running