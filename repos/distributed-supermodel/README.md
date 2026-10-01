# Distributed Supermodel

Proof of concept for an LLM partitioned into shards (pipeline + MoE) coordinated via libp2p.

## Planned components
- `coordinator/`: token routing, step sync, logging
- `shards/`: layer, expert, and MoE implementations
- `transport/`: libp2p/gRPC messaging
- `examples/`: scripts to run a 3-node local cluster

## Immediate goal
Run a simple forward pass across three nodes: Input → Layers 1-8 → Layers 9-16 → Experts. Capture tensors and latencies for public benchmarking.
