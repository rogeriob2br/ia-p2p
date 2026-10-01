/**
 * Getting Started documentation content
 * Feature: landing-page-documentation
 */

export const gettingStartedContent = `
# Getting Started

Welcome to InferencePool! This guide will help you get up and running in minutes.

## Prerequisites

Before you begin, make sure you have:

- **Node.js 18+** installed on your machine
- A package manager (\`npm\`, \`yarn\`, or \`pnpm\`)
- An InferencePool account with API key

## Installation

Install the InferencePool SDK using your preferred package manager:

\`\`\`bash
npm install @inference-pool/sdk
\`\`\`

Or with yarn:

\`\`\`bash
yarn add @inference-pool/sdk
\`\`\`

## Quick Start

Here's a complete example to get you started:

\`\`\`typescript
import { InferencePool } from '@inference-pool/sdk';

// Initialize the client
const pool = new InferencePool({
  apiKey: process.env.INFERENCE_POOL_API_KEY,
  network: 'mainnet'
});

// Make an inference request
const response = await pool.complete({
  model: 'gpt-4',
  messages: [
    { role: 'user', content: 'Hello, world!' }
  ]
});

console.log(response.choices[0].message.content);
\`\`\`

## Configuration Options

The InferencePool client accepts several configuration options:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| \`apiKey\` | string | - | Your API key (required) |
| \`network\` | string | 'mainnet' | Network to use |
| \`routing\` | string | 'weighted_score' | Routing strategy |
| \`timeout\` | number | 30000 | Request timeout in ms |

## Advanced Configuration

\`\`\`typescript
const pool = new InferencePool({
  apiKey: 'sk-pool-your-api-key-here',
  network: 'mainnet',
  routing: 'weighted_score', // or 'round_robin', 'least_loaded'
  timeout: 120000, // 2 minutes
  retries: 3,
  debug: true
});
\`\`\`

> **Note**: Never expose your API key in client-side code or public repositories. Use environment variables to store sensitive credentials.

## Next Steps

- Learn about [Authentication](/docs/getting-started/authentication) to secure your requests
- Explore the [API Reference](/docs/api-reference) for all available methods
- Set up [Lightning Payments](/docs/core-concepts/lightning-payments) for instant transactions
`;

export const installationContent = `
# Installation

Install the InferencePool SDK in your project.

## Package Managers

### npm

\`\`\`bash
npm install @inference-pool/sdk
\`\`\`

### yarn

\`\`\`bash
yarn add @inference-pool/sdk
\`\`\`

### pnpm

\`\`\`bash
pnpm add @inference-pool/sdk
\`\`\`

## TypeScript Support

The SDK includes TypeScript definitions out of the box. No additional \`@types\` package is needed.

\`\`\`typescript
import { InferencePool, type InferencePoolConfig } from '@inference-pool/sdk';

const config: InferencePoolConfig = {
  apiKey: process.env.INFERENCE_POOL_API_KEY!,
  network: 'mainnet'
};

const pool = new InferencePool(config);
\`\`\`

## Requirements

- Node.js 18 or later
- ES2020+ compatible environment
- For browser usage, a bundler like Vite or webpack

## Verifying Installation

Create a simple test file to verify the installation:

\`\`\`typescript
import { InferencePool } from '@inference-pool/sdk';

console.log('SDK version:', InferencePool.version);
console.log('Installation successful!');
\`\`\`

Run it with:

\`\`\`bash
npx tsx test.ts
\`\`\`
`;

export const quickStartContent = `
# Quick Start

Get started with InferencePool in less than 5 minutes.

## Step 1: Create an Account

1. Visit [inferencepool.ai](https://inferencepool.ai)
2. Click "Sign Up"
3. Verify your email address

## Step 2: Get Your API Key

1. Log in to your dashboard
2. Navigate to **Settings** → **API Keys**
3. Click **Create New Key**
4. Copy and store your key securely

## Step 3: Install the SDK

\`\`\`bash
npm install @inference-pool/sdk
\`\`\`

## Step 4: Make Your First Request

\`\`\`typescript
import { InferencePool } from '@inference-pool/sdk';

const pool = new InferencePool({
  apiKey: 'sk-pool-your-api-key'
});

const response = await pool.complete({
  model: 'gpt-4',
  messages: [
    { role: 'user', content: 'What is InferencePool?' }
  ]
});

console.log(response.choices[0].message.content);
\`\`\`

## What's Next?

- **Explore models**: See all [available models](/docs/api-reference/models)
- **Learn concepts**: Understand [P2P orchestration](/docs/core-concepts/p2p-orchestration)
- **Integrate**: Check [framework guides](/docs/integration-guides)
`;

export const authenticationContent = `
# Authentication

Secure your API requests with proper authentication.

## API Keys

All requests to InferencePool require a valid API key. Keys are used to:

- Identify your account
- Track usage and billing
- Enforce rate limits

## Creating an API Key

1. Log in to your [Dashboard](https://inferencepool.ai/dashboard)
2. Go to **Settings** → **API Keys**
3. Click **Create New Key**
4. Give it a descriptive name
5. Copy the key immediately (it won't be shown again)

## Using Your API Key

### SDK Authentication

\`\`\`typescript
import { InferencePool } from '@inference-pool/sdk';

const pool = new InferencePool({
  apiKey: process.env.INFERENCE_POOL_API_KEY
});
\`\`\`

### REST API Authentication

Include the key in the Authorization header:

\`\`\`bash
curl https://api.inferencepool.ai/v1/completions \\
  -H "Authorization: Bearer sk-pool-your-api-key-here" \\
  -H "Content-Type: application/json" \\
  -d '{"model": "gpt-4", "messages": [{"role": "user", "content": "Hello"}]}'
\`\`\`

## Security Best Practices

> **Warning**: Never expose your API key in client-side code!

1. **Use environment variables**
   \`\`\`bash
   export INFERENCE_POOL_API_KEY=sk-pool-your-key
   \`\`\`

2. **Add to .gitignore**
   \`\`\`
   .env
   .env.local
   \`\`\`

3. **Rotate keys regularly** - Create new keys periodically and revoke old ones

4. **Use separate keys** for development and production environments

## Rate Limits

| Plan | Requests/min | Tokens/day |
|------|-------------|------------|
| Free | 60 | 100,000 |
| Pro | 600 | 1,000,000 |
| Enterprise | Unlimited | Unlimited |
`;
