/**
 * Core Concepts documentation content
 * Feature: landing-page-documentation
 */

export const p2pOrchestrationContent = `
# P2P Orchestration

Learn how InferencePool orchestrates AI inference requests across a decentralized network.

## How It Works

InferencePool uses a peer-to-peer network to distribute inference requests across multiple providers. This approach offers several advantages:

1. **No single point of failure** - If one provider goes down, requests are automatically routed to others
2. **Competitive pricing** - Providers compete for requests, keeping costs low
3. **Geographic distribution** - Requests are routed to the nearest available provider

## Architecture Overview

\`\`\`
┌─────────────┐     ┌─────────────────────┐     ┌──────────────┐
│   Client    │────▶│  InferencePool SDK  │────▶│  P2P Network │
└─────────────┘     └─────────────────────┘     └──────────────┘
                              │                        │
                              ▼                        ▼
                    ┌─────────────────┐      ┌────────────────┐
                    │  Orchestrator   │      │   Providers    │
                    └─────────────────┘      │  ┌──────────┐  │
                                             │  │ Provider │  │
                                             │  │    A     │  │
                                             │  └──────────┘  │
                                             │  ┌──────────┐  │
                                             │  │ Provider │  │
                                             │  │    B     │  │
                                             │  └──────────┘  │
                                             └────────────────┘
\`\`\`

## Request Flow

1. **Client sends request** - Your application calls the SDK
2. **Orchestrator receives** - The request is encrypted and sent to the network
3. **Provider selection** - Based on routing strategy, a provider is selected
4. **Inference execution** - The provider runs the model and returns results
5. **Response delivery** - Results are returned to your application

## Code Example

\`\`\`typescript
import { InferencePool } from '@inference-pool/sdk';

const pool = new InferencePool({
  apiKey: process.env.INFERENCE_POOL_API_KEY,
  network: 'mainnet',
  routing: 'weighted_score'
});

// The SDK handles all P2P orchestration automatically
const response = await pool.complete({
  model: 'gpt-4',
  messages: [{ role: 'user', content: 'Hello!' }]
});
\`\`\`

## Network Stats

| Metric | Value |
|--------|-------|
| Active Providers | 150+ |
| Average Latency | 120ms |
| Uptime | 99.9% |
| Regions | 12 |
`;

export const lightningPaymentsContent = `
# Lightning Payments

InferencePool uses the Bitcoin Lightning Network for instant, low-cost payments.

## Why Lightning?

- **Instant settlements** - Payments confirm in milliseconds, not minutes
- **Micro-transactions** - Pay per token with fees under 1 sat
- **Global access** - No bank account or credit card required
- **Privacy** - Payments are not stored on a public blockchain

## How Payments Work

\`\`\`
Request ──▶ Inference ──▶ Invoice ──▶ Payment ──▶ Response
                              │
                              ▼
                      Lightning Network
\`\`\`

1. You make an inference request
2. The provider generates a Lightning invoice
3. Payment is sent instantly via Lightning
4. Response is delivered immediately

## Configuring Payments

\`\`\`typescript
const pool = new InferencePool({
  apiKey: process.env.INFERENCE_POOL_API_KEY,
  payment: {
    method: 'lightning',
    autoPayInvoices: true, // Automatically pay invoices
    maxSatsPerRequest: 1000 // Set spending limit
  }
});
\`\`\`

## Balance Management

Check your balance and top up easily:

\`\`\`typescript
// Check balance
const balance = await pool.getBalance();
console.log(\`Balance: \${balance.sats} sats\`);

// Get deposit address
const invoice = await pool.createInvoice({
  amount: 10000, // 10,000 sats
  description: 'Top up InferencePool balance'
});
console.log('Pay this invoice:', invoice.paymentRequest);
\`\`\`

## Pricing

| Model | Price per 1K tokens |
|-------|---------------------|
| GPT-4 | ~50 sats |
| GPT-3.5 | ~5 sats |
| Claude 3 | ~40 sats |
| Llama 70B | ~10 sats |

> **Tip**: Prices vary based on market conditions and provider availability.
`;

export const routingStrategiesContent = `
# Routing Strategies

Choose how InferencePool selects providers for your requests.

## Available Strategies

### Weighted Score (Default)

Balances price, latency, and reliability for optimal results.

\`\`\`typescript
const pool = new InferencePool({
  routing: 'weighted_score',
  routingOptions: {
    priceWeight: 0.4,    // 40% importance
    latencyWeight: 0.3,  // 30% importance
    reliabilityWeight: 0.3 // 30% importance
  }
});
\`\`\`

### Lowest Price

Always selects the cheapest available provider.

\`\`\`typescript
const pool = new InferencePool({
  routing: 'lowest_price'
});
\`\`\`

### Lowest Latency

Prioritizes the fastest response time.

\`\`\`typescript
const pool = new InferencePool({
  routing: 'lowest_latency'
});
\`\`\`

### Round Robin

Distributes requests evenly across providers.

\`\`\`typescript
const pool = new InferencePool({
  routing: 'round_robin'
});
\`\`\`

### Geolocation

Routes to the nearest provider based on location.

\`\`\`typescript
const pool = new InferencePool({
  routing: 'geolocation',
  routingOptions: {
    preferredRegions: ['us-east', 'eu-west']
  }
});
\`\`\`

## Custom Routing

Implement your own routing logic:

\`\`\`typescript
const pool = new InferencePool({
  routing: 'custom',
  customRouter: async (providers, request) => {
    // Your custom logic here
    const filtered = providers.filter(p => p.reputation > 0.9);
    return filtered.sort((a, b) => a.price - b.price)[0];
  }
});
\`\`\`

## Comparison

| Strategy | Best For | Trade-off |
|----------|----------|-----------|
| Weighted Score | General use | Balanced |
| Lowest Price | Cost-sensitive | May have higher latency |
| Lowest Latency | Real-time apps | May cost more |
| Round Robin | Load testing | No optimization |
| Geolocation | Global apps | Limited provider choice |
`;
