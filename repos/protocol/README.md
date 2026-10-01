# Protocol

Minimal schemas and examples for jobs, receipts, manifests, and reputation. This repo defines the common dialect every implementation must speak.

## Layout
- `schema/` JSON Schemas (`job`, `manifest`, `receipt`, `reputation`)
- `examples/` reference signed events (TBD)
- `docs/manifestos.md` manifesto guidelines
- `docs/handshake.md` network initialization flow
- `docs/sessions.md` streaming chat/session guidance

## Immediate tasks
1. Provide signed sample events referencing the schemas
2. Define Nostr kinds / topics mapping in `docs/`
3. Add automated schema validation scripts
