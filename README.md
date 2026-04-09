# Crowdfunding Program

A simple crowdfunding program built with Solana and Anchor.

This project allows:
- a `creator` to create a campaign with a `goal` and `deadline`
- `contributors` to send SOL into a program-controlled vault PDA
- the `creator` to `withdraw` funds if the goal is reached after the deadline
- `contributors` to `refund` their own contribution if the goal is not reached

Repository:
- https://github.com/geraltritama/crowdfunding

## Overview

The main on-chain program lives in `programs/crowdfunding/src/lib.rs`, and the integration test lives in `tests/crowdfunding.ts`.

flow:
1. The creator creates a campaign.
2. Contributors donate to the campaign.
3. After the deadline:
   - if `raised >= goal`, the creator can withdraw
   - if `raised < goal`, contributors can refund their own contribution

## Main Features

Required on-chain instructions:
- `create_campaign(goal: u64, deadline: i64)`
- `contribute(amount: u64)`
- `withdraw()`
- `refund()`

Additional instructions implemented in this repository:
- `close_contribution()`
- `reclaim_vault_rent()`

These extra instructions are used for account cleanup and rent recovery so lamports are not left behind unnecessarily.

## Account Design

Key accounts used by the program:
- `Campaign`: stores campaign data such as creator, goal, raised amount, deadline, and claim status
- `Vault PDA`: holds SOL for each campaign
- `Contribution PDA`: stores per-contributor contribution data for a specific campaign

Seeds:
- Vault PDA: `["vault", campaign]`
- Contribution PDA: `["contribution", campaign, contributor]`

With this design, `refund()` only returns the contributor's own recorded amount instead of allowing a user to drain the entire vault.

## Program ID

Configured program ID:
- `F3k9VhRhE9JwkgRNryjkzu7CL3z8epdLc3hHxe2Bg5gu`

This ID is registered in:
- `Anchor.toml` for `localnet`
- `Anchor.toml` for `devnet`
- `declare_id!` in the Rust program

## Devnet Deployment

Deploy/upgrade transaction signature:
- `4wnQSt5sBYhf4riRLpCzoyCwRU5EucpyEihz1z6gN64QaLYSqEF26skr975gQTeo4VP3YEXTKYpwAy9Ez5TpEJNp`

## Tech Stack

- Rust
- Anchor
- Solana CLI
- Node.js
- Yarn
- TypeScript
- Mocha + Chai

## Prerequisites

Make sure the following tools are available:
- Rust toolchain
- Solana CLI
- Anchor CLI
- Node.js
- Yarn
- a local Solana wallet, usually at `~/.config/solana/id.json`

## Setup

```bash
git clone https://github.com/geraltritama/crowdfunding
cd crowdfunding
yarn install
anchor build
```

## Running the Project

Build the program:

```bash
anchor build
```

Run the localnet runtime test:

```bash
yarn test:runtime
```

The `test:runtime` script will:
- start `solana-test-validator`
- build the program with Anchor
- deploy the program to localnet
- run the test in `tests/crowdfunding.ts`

## Testing

Main test file:
- `tests/crowdfunding.ts`

This test covers the two core flows:
- success flow: create -> contribute -> withdraw
- refund flow: create -> contribute -> refund

Example localnet signatures from a sample run:

Success flow:
- `create_campaign`: `2Ro8QpE1Pu7tfFXMQyZtYYXk8pT3A1bsojtaaQfXvXf3qQfEEenikiaQNFKB7n1APDDPsEn6Vm1D8Jju7ZQyNGyU`
- `contribute`: `2mbX1ZRec1ewgfr1srSib7Xkx5GyH2U14qYjECpR3Tx8BzAX8BwHdWbWTNUgqUDvBHG8X7yULwxYvAJFDa7pKd1r`
- `withdraw`: `2iw3ixxwnKvvn44MbY62euvWnix7Ny89dwkG8FzmXWCgsz4XjrXcQhV5HLSxNyKt4Baein6cSzjTWyj36LTyYwRm`

Refund flow:
- `create_campaign`: `3mmyobfLSYbuTcT7CcwKaNgVDU3JFStmEDd357mWHaz4bRy22jSUbD6R79mz4wN4XUyJ8SJKroTFTMnBwcPvt5Cx`
- `contribute`: `4awkrKecyR3ptqJfRXcgXBrB3uBMRiKvwzETFSBLvigPpepXnfzK8PbbXMUvbcaod9ScFhhR6vtzPfj4taSgjUNP`
- `refund`: `3ufwhFf9fJZ69df8Le2RXyCM3uRHn7jLPwu2A7MfQjf9K7WptjtK9FXtjYJNbsVrrKNoKvfKaTbvWwavaRBnzo1W`

## Project Structure

Main project layout:

```text
crowdfunding/
├── Anchor.toml
├── Cargo.toml
├── package.json
├── tsconfig.json
├── yarn.lock
├── README.md
├── app/
├── migrations/
│   └── deploy.ts
├── programs/
│   └── crowdfunding/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── tests/
│   └── crowdfunding.ts
├── target/
└── test-ledger/
```

Short explanation:
- `Anchor.toml`: Anchor workspace, provider, and program ID configuration
- `Cargo.toml`: root Rust workspace configuration
- `package.json`: JavaScript/TypeScript dependencies and scripts
- `migrations/`: Anchor deploy script
- `programs/crowdfunding/`: on-chain program source code
- `tests/`: TypeScript integration tests
- `app/`: placeholder for a client app, currently empty
- `target/`: Anchor build output and generated artifacts
- `test-ledger/`: local validator ledger data from testing

## Implementation Notes

Important implementation details:
- the vault is created as a system-owned PDA account
- contributions are tracked per contributor through the `Contribution PDA`
- `withdraw()` is only available to the creator after the deadline if the goal is reached
- `refund()` is only available after the deadline if the goal is not reached
- extra cleanup is handled through `close_contribution()` and `reclaim_vault_rent()`