# Network Handshake & Initialization

This document describes how a fresh node joins the network and starts receiving jobs.

## 1. Local bootstrap
- Generate or import Ed25519 keys; store under `~/.p2p-ai/keys`.
- Create `manifest.local.json` describing capabilities (hardware, execution modes, payment preferences).
- Configure transports: list of Nostr relays, torrent trackers/Bitswap peers, bootstrap libp2p peers.

## 2. Publishing availability
1. Connect to at least two Nostr relays.
2. Publish an event `kind=job-provider` containing:
   - `pubkey`
   - hash of the local manifest
   - torrent/libp2p endpoints
   - software version, timestamp, signature
3. Optionally listen for ACK/feedback events.

## 3. Syncing manifests
- Subscribe to `kind=manifest-announce`.
- Download referenced manifests (Nostr/IPFS), verify signatures, cache locally.
- Only accept jobs whose `policy_refs` match installed manifests.

## 4. Job & session intake
1. Subscribe to `kind=job-offer` (batch) and `kind=session-open` (streaming) events.
2. For each offer evaluate:
   - hardware requirements vs capabilities
   - policy references
   - reward/payment terms
3. If acceptable, publish `kind=job-claim` (batch) or `kind=session-ack` (streaming) including job/session identifiers, executor pubkey, timestamp, signature.
4. Begin provisioning (download artifacts via torrent/Bitswap, prepare runtime, establish libp2p channels).

## 5. Execution & receipts
- Run the job/session according to manifest rules.
- Batch jobs: emit a single receipt referencing the job and manifest, following `receipt.schema.json`.
- Sessions: emit micro-receipts per token window (`session_id`, `sequence`, `chunk_window_seconds`), then optionally publish a final aggregated receipt.
- Include hashes of outputs, runtime metrics, and optional payment proofs.

## 6. Payments & reputation
- Execute the payment rail specified by the job (Lightning, NWC, fiat settlement).
- Monitor `kind=reputation-update` events. Publish feedback when validating peers (score, reason, evidence URI) following `reputation.schema.json`.

Following this flow ensures any new node is discoverable, policy-aware, and ready to process jobs without manual coordination.
