# What is MTU Optimizer?

A command-line tool that finds the optimal MTU (Maximum Transmission Unit) size for your network connection by testing different values and analyzing latency, jitter, and packet loss.

### Features

- **Dynamic Progress Bar**: Real-time progress tracking during MTU testing
- **Comprehensive Analysis**: Latency (min/avg/max), jitter, and packet loss statistics
- **Clean Results**: Identifies optimal and worst-performing MTU values
- **Flexible Output**: Text table or JSON format
- **Customizable Testing**: Configurable MTU range, step size, and ping count

## Quick Start

```bash
# Build the application
make build

# Run with default settings (tests MTU 1200-1500)
./build/mtu

# Test specific range
./build/mtu --min-mtu 1400 --max-mtu 1460 --step 10

# Verbose output with detailed information
./build/mtu --verbose --requests 3
```

## Installation

```bash
# Install dependencies and build
make deps
make build
```

## Usage Examples

### Basic Optimization
```bash
./build/mtu
```
Tests MTU range 1200-1500 with step 8, showing progress bar and results table.

### Custom Target and Range
```bash
./build/mtu --target 1.1.1.1 --min-mtu 1400 --max-mtu 1500 --step 20
```

### Fast Testing
```bash
./build/mtu --min-mtu 1450 --max-mtu 1470 --step 5 --requests 2
```

### JSON Output
```bash
./build/mtu --format json --min-mtu 1400 --max-mtu 1450 --step 10
```

### Verbose Mode
```bash
./build/mtu --verbose --debug
```

## Configuration Options

| Flag | Description | Default |
|------|-------------|---------|
| `--target` | Target IP address for ping tests | 8.8.8.8 |
| `--min-mtu` | Minimum MTU size to test | 1200 |
| `--max-mtu` | Maximum MTU size to test | 1500 |
| `--step` | Increment between MTU tests | 8 |
| `--requests` | Number of pings per MTU size | 5 |
| `--timeout-ms` | Timeout per ping (milliseconds) | 3000 |
| `--format` | Output format (`text`, `json`, `csv`) | text |
| `--verbose` | Show detailed progress information | false |
| `--debug` | Enable debug output | false |
| `--skip-connectivity-test` | Skip initial connectivity check | false |

## License

[![LICENSE - MIT by gvrciary](https://img.shields.io/badge/LICENSE-MIT-111111?style=for-the-badge&labelColor=111111&logo=open-source-initiative&logoColor=white)](LICENSE)