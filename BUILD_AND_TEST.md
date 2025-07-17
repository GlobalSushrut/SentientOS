# SentientOS Build and Test Guide

This document provides comprehensive instructions for building SentientOS from source and running various tests, including the Oracle burn test suite.

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Building SentientOS](#building-sentientos)
3. [Running Basic Tests](#running-basic-tests)
4. [Oracle Burn Test](#oracle-burn-test)
5. [Performance Benchmarks](#performance-benchmarks)
6. [Troubleshooting](#troubleshooting)

## Prerequisites

Before building SentientOS, ensure you have the following dependencies installed:

- Rust 1.69.0 or newer
- Zig 0.10.0 or newer
- WASM toolchain
- TinyGo 0.27.0 or newer
- Blake3 library
- CMake 3.22 or newer
- LLVM 15.0 or newer
- ZK proving tools (for full ZK functionality)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Zig
wget https://ziglang.org/download/0.10.0/zig-linux-x86_64-0.10.0.tar.xz
tar -xf zig-linux-x86_64-0.10.0.tar.xz
export PATH=$PATH:$(pwd)/zig-linux-x86_64-0.10.0

# Install WASM toolchain
rustup target add wasm32-wasi
rustup target add wasm32-unknown-unknown

# Install TinyGo
wget https://github.com/tinygo-org/tinygo/releases/download/v0.27.0/tinygo_0.27.0_amd64.deb
sudo dpkg -i tinygo_0.27.0_amd64.deb
```

## Building SentientOS

### Standard Build

To build SentientOS with default features:

```bash
# Clone the repository if you haven't already
git clone https://github.com/sentient-os/sentientos.git
cd sentientos

# Build the project
cargo build --release
```

### Custom Build Configurations

For specialized builds:

```bash
# Minimal build without ZK verification (faster but less secure)
cargo build --release --no-default-features --features minimal

# Full build with all features
cargo build --release --features full,telemetry,plugins

# Build for IoT devices
cargo build --release --target thumbv7em-none-eabihf --features iot
```

## Running Basic Tests

Run the test suite to verify basic functionality:

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test --test zk_tests
cargo test --test matrixbox_tests
cargo test --test gossip_tests
```

## Oracle Burn Test

The Oracle burn test is a comprehensive stress test that verifies the system under extreme conditions.

### Setting Up Oracle Burn Test

```bash
# Install Oracle burn test dependencies
cd tools/oracle-burn
cargo build --release

# Prepare test environment
./setup_test_env.sh
```

### Running Oracle Burn Tests

```bash
# Run full Oracle burn test suite (takes approximately 2 hours)
./oracle-burn --full

# Run specific test categories
./oracle-burn --zk-stress
./oracle-burn --container-stress
./oracle-burn --sync-stress

# Run recovery test
./oracle-burn --panic-recovery
```

The Oracle burn test runs through the following phases:

1. **Bootstrap Testing**: Verifies system initialization
2. **ZK Proof Overload**: Generates and verifies thousands of ZK proofs
3. **Container Stress**: Launches and terminates containers rapidly
4. **Sync Chaos**: Creates network partition scenarios and tests sync
5. **Panic Recovery**: Forces system panics and verifies recovery
6. **Intent Replay**: Records and replays complex intent sessions
7. **Store Package Flood**: Installs and removes packages rapidly

### Oracle Burn Test Success Criteria

The test suite generates a report in `./burn-report.json` with the following key metrics:

- Zero-knowledge verification time
- Recovery time after forced panic
- Container startup/shutdown latency
- Sync convergence time
- Memory usage under load
- CPU utilization patterns

Success requires all tests passing with metrics within the defined thresholds.

## Performance Benchmarks

Run performance benchmarks to measure system efficiency:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark groups
cargo bench --bench zk_bench
cargo bench --bench container_bench
```

Key performance targets:
- ZK verification: <50ms per proof
- Container startup: <100ms
- Full system boot: <3 seconds
- Package installation: <2 seconds
- Memory footprint: <100MB idle

## Troubleshooting

### Common Build Issues

**Problem**: Missing Zig dependency
**Solution**: Verify Zig is in PATH and version is 0.10.0 or newer

**Problem**: ZK verification failures
**Solution**: Check that ZK proving tools are properly installed

**Problem**: "cargo not found" errors
**Solution**: Ensure Rust is properly installed with `source $HOME/.cargo/env`

### Log Analysis

Logs are stored in `.sentinel/logs/` with the following categories:

- `zk.log`: Zero-knowledge verification logs
- `matrixbox.log`: Container operation logs
- `intent.log`: Developer intent records
- `panic.log`: System panic and recovery events

Use the log inspector to analyze logs:

```bash
./tools/log-inspector --file .sentinel/logs/zk.log
```

### Getting Help

For additional assistance, refer to:
- GitHub issues: https://github.com/sentient-os/sentientos/issues
- Documentation: https://docs.sentientos.org/
- Community forum: https://community.sentientos.org/

## License

SentientOS is open-source software licensed under the MIT license.
