/**
 * API Reference documentation content
 * Feature: landing-page-documentation
 */

export const clientContent = `
# InferencePool Client

The main client class for interacting with the InferencePool network.

## Constructor

\`\`\`typescript
import { InferencePool } from '@inference-pool/sdk';

const pool = new InferencePool(config: InferencePoolConfig);
\`\`\`

## Configuration

\`\`\`typescript
interface InferencePoolConfig {
  // Required
  apiKey: string;
  
  // Optional
  network?: 'mainnet' | 'testnet';
  routing?: RoutingStrategy;
  timeout?: number;
  retries?: number;
  debug?: boolean;
  payment?: PaymentConfig;
}
\`\`\`

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| \`apiKey\` | \`string\` | - | Your API key (required) |
| \`network\` | \`string\` | \`'mainnet'\` | Network to connect to |
| \`routing\` | \`string\` | \`'weighted_score'\` | Provider selection strategy |
| \`timeout\` | \`number\` | \`30000\` | Request timeout in milliseconds |
| \`retries\` | \`number\` | \`3\` | Number of retry attempts |
| \`debug\` | \`boolean\` | \`false\` | Enable debug logging |

## Example

\`\`\`typescript
const pool = new InferencePool({
  apiKey: process.env.INFERENCE_POOL_API_KEY,
  network: 'mainnet',
  routing: 'weighted_score',
  timeout: 60000,
  retries: 3,
  debug: true
});
\`\`\`
`;

export const methodsContent = `
# Methods

Complete reference for all SDK methods.

## complete()

Generate text completions from AI models.

\`\`\`typescript
const response = await pool.complete(options: CompletionOptions);
\`\`\`

### Parameters

\`\`\`typescript
interface CompletionOptions {
  model: string;
  messages: Message[];
  temperature?: number;
  maxTokens?: number;
  stream?: boolean;
}
\`\`\`

### Response

\`\`\`typescript
interface CompletionResponse {
  id: string;
  choices: Choice[];
  usage: {
    promptTokens: number;
    completionTokens: number;
    totalTokens: number;
  };
  provider: string;
  cost: number; // in satoshis
}
\`\`\`

### Example

\`\`\`typescript
const response = await pool.complete({
  model: 'gpt-4',
  messages: [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'Explain quantum computing.' }
  ],
  temperature: 0.7,
  maxTokens: 500
});

console.log(response.choices[0].message.content);
\`\`\`

---

## stream()

Stream completions for real-time responses.

\`\`\`typescript
const stream = await pool.stream(options: CompletionOptions);

for await (const chunk of stream) {
  process.stdout.write(chunk.content);
}
\`\`\`

---

## embed()

Generate embeddings for text.

\`\`\`typescript
const embeddings = await pool.embed({
  model: 'text-embedding-ada-002',
  input: 'Hello, world!'
});

console.log(embeddings.data[0].embedding);
\`\`\`

---

## getModels()

List available models.

\`\`\`typescript
const models = await pool.getModels();

for (const model of models) {
  console.log(\`\${model.id}: \${model.pricePerToken} sats/token\`);
}
\`\`\`

---

## getBalance()

Check your account balance.

\`\`\`typescript
const balance = await pool.getBalance();
console.log(\`Balance: \${balance.sats} sats (\${balance.usd} USD)\`);
\`\`\`
`;

export const configurationContent = `
# Configuration

Advanced configuration options for the InferencePool SDK.

## Environment Variables

\`\`\`bash
# Required
INFERENCE_POOL_API_KEY=sk-pool-your-key

# Optional
INFERENCE_POOL_NETWORK=mainnet
INFERENCE_POOL_TIMEOUT=30000
INFERENCE_POOL_DEBUG=false
\`\`\`

## Configuration File

Create \`inferencepool.config.js\` in your project root:

\`\`\`javascript
module.exports = {
  apiKey: process.env.INFERENCE_POOL_API_KEY,
  network: 'mainnet',
  routing: {
    strategy: 'weighted_score',
    weights: {
      price: 0.4,
      latency: 0.3,
      reliability: 0.3
    }
  },
  payment: {
    autoPayInvoices: true,
    maxSatsPerRequest: 1000
  },
  retry: {
    attempts: 3,
    backoff: 'exponential',
    maxDelay: 10000
  }
};
\`\`\`

## TypeScript Configuration

The SDK includes full TypeScript support:

\`\`\`typescript
import { 
  InferencePool, 
  InferencePoolConfig,
  CompletionOptions,
  CompletionResponse 
} from '@inference-pool/sdk';

const config: InferencePoolConfig = {
  apiKey: process.env.INFERENCE_POOL_API_KEY!,
  network: 'mainnet'
};

const pool = new InferencePool(config);
\`\`\`
`;

export const errorsContent = `
# Error Handling

Handle errors gracefully in your InferencePool integration.

## Error Types

\`\`\`typescript
import { 
  InferencePoolError,
  AuthenticationError,
  RateLimitError,
  NetworkError,
  PaymentError,
  ProviderError
} from '@inference-pool/sdk';
\`\`\`

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| \`AUTH_001\` | Invalid API Key | The API key is invalid or expired |
| \`AUTH_002\` | Insufficient Permissions | Key lacks required permissions |
| \`RATE_001\` | Rate Limit Exceeded | Too many requests |
| \`NET_001\` | Network Timeout | Request timed out |
| \`NET_002\` | Connection Failed | Unable to connect to network |
| \`PAY_001\` | Insufficient Balance | Not enough sats |
| \`PAY_002\` | Payment Failed | Lightning payment failed |
| \`PROV_001\` | No Providers | No providers available |
| \`PROV_002\` | Provider Error | Provider returned an error |

## Handling Errors

\`\`\`typescript
import { InferencePool, InferencePoolError, RateLimitError } from '@inference-pool/sdk';

try {
  const response = await pool.complete({
    model: 'gpt-4',
    messages: [{ role: 'user', content: 'Hello' }]
  });
} catch (error) {
  if (error instanceof RateLimitError) {
    console.log(\`Rate limited. Retry after \${error.retryAfter}ms\`);
    await sleep(error.retryAfter);
    // Retry the request
  } else if (error instanceof InferencePoolError) {
    console.error(\`Error \${error.code}: \${error.message}\`);
  } else {
    throw error;
  }
}
\`\`\`

## Retry Configuration

\`\`\`typescript
const pool = new InferencePool({
  apiKey: process.env.INFERENCE_POOL_API_KEY,
  retry: {
    attempts: 5,
    backoff: 'exponential',
    maxDelay: 30000,
    retryOn: ['RATE_001', 'NET_001', 'PROV_001']
  }
});
\`\`\`
`;
