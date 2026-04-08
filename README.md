# Crowdfunding (Solana + Anchor)

A simple crowdfunding program:
- `creator` creates a campaign with a `goal` and `deadline`
- `contributors` donate SOL into a program-controlled vault PDA
- if the goal is met after the deadline, the creator can `withdraw`
- otherwise, contributors can `refund` only their own contribution

Repo: https://github.com/geraltritama/crowdfunding

## Instructions (on-chain API)

Per `kriteria.md`, the program exposes **4 instructions**:
- `create_campaign(goal: u64, deadline: i64)`
- `contribute(amount: u64)`
- `withdraw()` (creator claim on success)
- `refund()` (contributor refund on failure)

## Security-critical design (high level)

- **Vault PDA** (escrow): seeds = `["vault", campaign]`
- **Contribution PDA** (per contributor): seeds = `["contribution", campaign, contributor]`

Refund uses the contributor’s `Contribution` PDA, preventing a user from draining the entire vault.

## Devnet Deployment (for submission)

Program ID:
- `F3k9VhRhE9JwkgRNryjkzu7CL3z8epdLc3hHxe2Bg5gu`

Deploy/upgrade transaction signature:
- `4wnQSt5sBYhf4riRLpCzoyCwRU5EucpyEihz1z6gN64QaLYSqEF26skr975gQTeo4VP3YEXTKYpwAy9Ez5TpEJNp`

## Prerequisites

- Rust + Anchor
- Node.js + Yarn
- Solana CLI configured to have access to your keypair (usually `~/.config/solana/id.json`)

## Setup

```bash
git clone https://github.com/geraltritama/crowdfunding
cd crowdfunding
yarn install
anchor build
```

## Build

```bash
anchor build
```

## Tests / Evidence

### TypeScript (IDL structure & PDA seed checks)

This evidence verifies security-critical constraints from the generated IDL:
- `create_campaign` exposes vault PDA seeds `["vault", campaign]`
- `refund` includes the per-user `Contribution` PDA seeds `["contribution", campaign, contributor]`

Run:

```bash
yarn test:ts
```

The test file is:
- `tests/crowdfunding.ts`

## Files of interest (for code review)

- Program entrypoint: `programs/crowdfunding/src/lib.rs`
- TS evidence test: `tests/crowdfunding.ts`
