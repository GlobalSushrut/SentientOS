# SentientOS TSO Burn

This is a full TSO burn of SentientOS, created on 2025-07-20T04:51:39Z.

## Running the OS

To start SentientOS in TSO mode, run:

```
./sentient.sh
```

## Components

- ZK-YAML contracts in `zk_contracts/`
- MatrixBox containers in `.container/`
- Linux compatibility layer in `.linux/`
- System components in `.runtime/`, `.lock/`, etc.

## Architecture

This TSO burn follows the architecture described in the SentientOS planning document,
implementing the Oracle Partial Runtime that runs within an existing OS but provides
a complete SentientOS environment.
