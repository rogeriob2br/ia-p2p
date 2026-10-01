# Whitepaper: Decentralized Autonomous AI Network

Version 0.3 – Working Draft

## 1. Introduction

We live in a moment where a handful of groups control access to frontier AI. They decide who can train, who can consume, and what the price is. This proposal breaks that logic: build an open network where anyone can plug in a GPU, CPU or accelerator, run living models, and receive purely peer-to-peer payments. Autonomy does not depend on a company; it stems from a simple, auditable, forkable protocol inspired by the original spirit of Bitcoin and public networks.

## 2. Motivation and Reality

Closed models concentrate power, enforce soft censorship, and waste idle hardware scattered across the planet. Meanwhile, GPU marketplaces and P2P networks prove the appetite for decentralized solutions, but most remain corporate hybrids or rely on proprietary tokens. This network exists for those who need inexpensive inference, who want to micro-train on demand, and who prefer to convert electricity into useful intelligence without asking for permission. The goal is not another “service,” but an ecosystem where efficiency and honesty are rewarded at the edge instead of the center.

## 3. Actors and Incentives

The network exists because every participant pursues self-interest and, by doing so, reinforces the system. End users show up with concrete demands—private chats, simulations, bespoke micro-training—and broadcast jobs without requesting approval. They pay strictly for what was delivered. On the other side are compute providers, the miners of this realm, who aim their GPUs, CPUs, or accelerators at the most lucrative jobs and spend the day tuning quantization, batching, and cooling to squeeze more satoshis per watt. Between them emerge execution pools: spontaneous cooperatives that bundle heterogeneous hardware, absorb variance risk, enforce cross-checks, and redistribute rewards according to proven work.

Relays, explorers, and observers round out the picture. They command no one but supply the connective tissue: indexing announcements, replicating manifests, storing receipts, and calculating public reputation. Much like Bitcoin full nodes, these agents make history auditable and allow any participant to detect censorship or fraud.

In the end, incentives align brutally: good service yields more jobs; deception erodes reputation and revenue; efficiency breakthroughs win entire pools. There are no regulatory shortcuts or monopolies—only open competition for payment tied to useful work, guided by the same instincts that power Bitcoin mining, now applied to distributed intelligence.

## 4. Protocol Core

The protocol defines only what is necessary for everyone to talk. Job, receipt, and reputation events carry required fields (identifiers, model hashes, hardware requirements, payment terms) signed by public keys. Those events may travel over any transport layer—Nostr, libp2p, or another—as long as they keep the shared format. From that nucleus, each group publishes manifests describing its own policies: latency targets, redundancy, permitted models, jurisdictions. Manifests are merely signed documents on the same network, allowing organic forks at any time. No authority approves or rejects a fork; the market’s natural selection does. Because every manifest references the same base dialect, a node can participate in multiple contexts with no friction.

## 5. Hybrid Execution Layer

The network accepts two complementary execution families:

1. **Specialized micromodels** – compact models (up to ~7B parameters, quantized) hosted on entry-level GPUs (gaming boards, integrated graphics) and on dedicated edge devices such as NVIDIA Jetson. They excel at low-latency, vertical tasks. A lightweight router classifies the intent and dispatches the best micromodel. This arrangement simplifies onboarding and keeps compute geographically close to the users.
2. **Distributed supermodel** – a large model partitioned into shards (pipeline, MoE, context parallelism). Each node keeps a slice of the model or a set of experts. The forward pass is coordinated via libp2p, with intermediate tensors flowing between trusted nodes. This brings “planetary” capacity, enabling models larger than any single piece of hardware could host.

Both paths are detailed in `docs/distributed-models.md`. Pools may operate micromodels, supermodel shards, or any mix, yet all follow the same protocol for publishing jobs and collecting receipts. Every node runs a lightweight agent that detects its capabilities (vendor, VRAM, bandwidth, energy cost) and advertises manifests describing which execution modes it supports. When a compatible job arrives, the agent fetches the appropriate container (CUDA, ROCm, llama.cpp, TensorRT, ONNX, GGUF) and reports outputs in the common format. Competition among pools drives constant optimization in caching, quantization, tensor compression, and pipeline layouts.

## 6. Economic Layer and Payments

Payments travel over genuinely open rails: Lightning Network, Nostr Wallet Connect, federated ecash, or direct fiat settlement. Each job states the intended reward and settlement method; providers choose whether to accept. Micropayments may stream continuously, close per batch, or operate via escrow, and the protocol leaves room to split fees (executor, pool, relay, maintenance). There is no native token nor ICO: trust relies on verifiable settlement and public reputation. Prices adapt to supply and demand: regions that demand ultralow latency pay more; rare model expertise commands a premium. This free market replaces Bitcoin’s difficulty adjustment, keeping the network alive as external conditions shift.

## 7. Validation, Reputation, and Security

There is no global block consensus; only local consensus on who delivered what. Each job can request redundancy, canary tasks, TEEs, or specific cryptographic proofs, and manifests may impose distinct combinations. Signed receipts include output hashes, execution metadata, elapsed time, and proof of payment. Any relay can verify, archive, and publish reputation data, and any fork decides how much weight to assign each source. Malicious nodes face economic penalties: they lose reputation, receive fewer jobs, and may forfeit bonds when canaries detect fraud. Because reputation lives in the open, independent auditors can expose pools that fake results or manipulate data. Censorship becomes unlikely: jobs can land on any relay and be re-broadcast by others.

## 8. Seed Model and Continuous Evolution

The network launches with an open-weight model acting as shared DNA. It is trained or tuned using public data (for example, Nostr), respecting permissive licenses and anonymization policies. Distributed micro-training occurs in short cycles: a job can dispatch batches to multiple nodes, collect gradients (or LoRA deltas), and merge them in a public, versioned repository. Competing model lineages may emerge organically; each manifest chooses which lineage to adopt. Beyond the supermodel, the community encourages specialized micromodels whose code and weights remain available so anyone can spawn their own evolutionary branch. The prime directive is reproducibility: datasets, scripts, weights, and hyperparameters stay in public repos, allowing audit and parallel innovation.

## 9. Conclusion and Next Steps

This network does not seek total consensus, only minimal compatibility and maximal freedom. Any fork is welcome if it speaks the common dialect. Any node is welcome if it proves useful work. Any user is welcome if they pay transparently. The immediate next step is straightforward: release the open-source code for the universal agent, the micromodel router, and the distributed supermodel prototype so anyone can test and demonstrate real proof-of-work. Formal documents follow later, as a byproduct of usage. First we ship practical evidence—public benchmarks, reproducible scripts, Lightning payment logs—and let the community run, critique, and fork. From there, the market decides which manifests persist. The protocol stays simple; evolution remains inevitably complex and organic.
