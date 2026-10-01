# Universal Agent

Rust-based agent that detects hardware, announces capabilities, and executes jobs for the decentralized AI network.

## Commands (WIP)
- `universal-agent detect [--gpu-vendors auto|none|nvidia]` — prints local hardware inventory in JSON
- `universal-agent init [--force] [--gpu-vendors ...] [--secret-key <nsec/hex>]` — generates or imports keypair + local manifest under `~/.p2p-ai`
- `universal-agent announce [--relay wss://...]` — publishes a `job-provider` event via Nostr relay

## Build
```
cargo run -- detect
cargo run -- init
cargo run -- announce

# With NVIDIA GPU detection
cargo run --features nvidia -- detect --gpu-vendors nvidia
```
