# Crowdfunding 

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

### TypeScript (runtime flow on localnet)

This runs an end-to-end flow on a local validator (create → contribute → withdraw/refund checks).

Run:

```bash
yarn test:runtime
```

The test file is:
- `tests/crowdfunding.ts`

Example localnet signatures (from a sample run):
- success flow:
  - `create_campaign`: `2Ro8QpE1Pu7tfFXMQyZtYYXk8pT3A1bsojtaaQfXvXf3qQfEEenikiaQNFKB7n1APDDPsEn6Vm1D8Jju7ZQyNGyU`
  - `contribute`: `2mbX1ZRec1ewgfr1srSib7Xkx5GyH2U14qYjECpR3Tx8BzAX8BwHdWbWTNUgqUDvBHG8X7yULwxYvAJFDa7pKd1r`
  - `withdraw` (after deadline): `2iw3ixxwnKvvn44MbY62euvWnix7Ny89dwkG8FzmXWCgsz4XjrXcQhV5HLSxNyKt4Baein6cSzjTWyj36LTyYwRm`
- refund flow:
  - `create_campaign`: `3mmyobfLSYbuTcT7CcwKaNgVDU3JFStmEDd357mWHaz4bRy22jSUbD6R79mz4wN4XUyJ8SJKroTFTMnBwcPvt5Cx`
  - `contribute`: `4awkrKecyR3ptqJfRXcgXBrB3uBMRiKvwzETFSBLvigPpepXnfzK8PbbXMUvbcaod9ScFhhR6vtzPfj4taSgjUNP`
  - `refund` (after deadline): `3ufwhFf9fJZ69df8Le2RXyCM3uRHn7jLPwu2A7MfQjf9K7WptjtK9FXtjYJNbsVrrKNoKvfKaTbvWwavaRBnzo1W`

## Files of interest (for code review)

- Program entrypoint: `programs/crowdfunding/src/lib.rs`
- TS evidence test: `tests/crowdfunding.ts`
