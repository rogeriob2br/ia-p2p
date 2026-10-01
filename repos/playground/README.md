# Playground

Demonstration and benchmarking environment tying together the protocol, agent, micromodels, distributed supermodel, and vector memory.

## Suggested structure
- `scenarios/`: YAML descriptions of test topologies
- `scripts/`: automation to spin up containers and collect metrics
- `benchmarks/`: published results (latency, throughput, cost)

## Initial proof-of-work
1. Run the “micromodel” scenario (client → router → agent) with simulated payments
2. Run the “supermodel” scenario with three local nodes
3. Publish logs/receipts in the repo for audit
