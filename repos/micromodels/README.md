# Micromodels

Router plus a suite of compact models ready to run on entry-level GPUs and edge devices.

## Planned structure
- `router/` (FastAPI + intent classifier)
- `models/<name>/` configs and download/quantization scripts
- `deploy/` docker-compose/k8s for pools
- `benchmarks/` public metrics

## First deliverables
1. Simple router forwarding to local models
2. Quantized 7B model (GGUF) with inference script
3. Documentation for running on Jetson and gaming GPUs
