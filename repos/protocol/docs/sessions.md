# Session-based Workflows

LLM chats and other streaming interactions rely on long-lived sessions instead of single batch jobs. This document describes the events and schemas involved.

## 1. Session open
- Event `kind=session-open`
- References `job_id`, `session.session_id`, manifest list, payment mode.
- Published by the client to announce intent and reserve resources.

## 2. Message exchange
- `kind=session-msg`: user input fragment, includes `session_id`, `sequence`, optional hash of accumulated context.
- `kind=session-reply`: provider response fragment, symmetric structure.
- Messages are lightweight and do not carry full receipts.

## 3. Micro-receipts
- Providers emit `kind=session-receipt` referencing `session_id` and `sequence`.
- Payload follows `receipt.schema.json` with `session_id`, `sequence`, `chunk_window_seconds`, and token counts for that window.
- Payment proofs (Lightning updates, NWC invoices) can be attached per micro-receipt.

## 4. Session heartbeat & close
- Heartbeat (`kind=session-heartbeat`) keeps the session alive; interval defined in the job’s `session` block.
- Either side can publish `kind=session-close` indicating graceful termination or error.

## 5. Aggregated settlement
- Optionally, the provider publishes a final aggregated receipt referencing all sequences for auditing.

These events allow interactive workloads to reuse the same protocol primitives (jobs, manifests, receipts) while supporting streaming UX.
