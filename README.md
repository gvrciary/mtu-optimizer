# MTU Checker

A basic **MTU Checker** developed in Rust that discovers the optimal MTU (Maximum Transmission Unit) size for your network connection through ping latency tests to a target host.

- **Automatic optimal MTU discovery** based on lowest average latency
- **Concurrent testing** for faster results
- **Detailed statistics** including packet loss, latency ranges and success rates
- **Highly configurable** with complete CLI options
- **Robust error handling** with detailed error messages
- **Visual progress bar** during testing
- **Graceful signal handling** for interruption

## Usage

### Basic Usage

```bash
# Use default configuration (ping 8.8.8.8, MTU range 1200-1500)
mtu-checker

# Specify custom target
mtu-checker --target 1.1.1.1

# Use custom MTU range
mtu-checker --min-mtu 1300 --max-mtu 1600
```

### Advanced Usage

```bash
# Complete configuration
mtu-checker \
    --target 8.8.8.8 \
    --min-mtu 1200 \
    --max-mtu 1500 \
    --step 8 \
    --requests 10 \
    --timeout-ms 5000 \
    --verbose

# JSON output for automation
mtu-checker --format json > mtu_results.json

# Faster testing with less precision
mtu-checker --requests 3 --timeout-ms 1000 --step 50

# Skip initial connectivity test
mtu-checker --skip-connectivity-test
```

## Configuration Options

| Option | Description | Default value |
|--------|-------------|---------------|
| `--target` | Target IP address for ping | `8.8.8.8` |
| `--min-mtu` | Minimum MTU size to test | `1200` |
| `--max-mtu` | Maximum MTU size to test | `1500` |
| `--step` | Increment between MTU tests | `8` |
| `--requests` | Number of pings per MTU size | `5` |
| `--timeout-ms` | Timeout per ping in milliseconds | `3000` |
| `--interface` | Network interface to use | `auto` |
| `--format` | Output format (text, json) | `text` |
| `--verbose` | Enable verbose output | `false` |
| `--debug` | Enable debug output | `false` |
| `--skip-connectivity-test` | Skip initial connectivity test | `false` |
| `--max-concurrent` | Maximum concurrent operations | `10` |

## Result Interpretation

### Status Symbols
- **●** (green): Successful ping
- **●** (red): Failed ping
- **○**: Ping with timeout

### Optimal MTU Selection Criteria
1. **Lowest average latency** (primary criterion)
2. **High success rate** (>90% recommended)
3. **Largest MTU size** in case of latency tie

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for more details.