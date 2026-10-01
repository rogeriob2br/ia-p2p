# Distributed Vector Memory

Pipeline for generating, packaging, and distributing embeddings/indexes for decentralized RAG.

## Planned components
- `builder/`: scripts to create embeddings and FAISS/HNSW indexes
- `shards/`: standardized format (metadata, vectors, signatures)
- `torrent/`: tooling to publish via BitTorrent/libp2p Bitswap
- `query-node/`: API serving searches from local shards

## First milestones
1. Produce a sample shard from a public dataset
2. Publish manifest + torrent
3. Spin up a query-node that fetches shards on demand and answers queries
