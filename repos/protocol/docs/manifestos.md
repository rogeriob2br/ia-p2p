# Execution Manifests

Each manifesto describes optional rules (latency, redundancy, allowed models). Suggested publication flow:
1. Generate JSON with fields `id`, `version`, `rules`, `contact`.
2. Sign with the operator key.
3. Publish through a Nostr/libp2p event (kind to be specified).

Implementations must accept multiple manifests simultaneously.
