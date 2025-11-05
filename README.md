# Ticket Vault – NFT, SOL & SPL Token Staking on Solana

> **Secure, flexible, and reward-generating multi-asset staking vault for NFTs, SOL, and SPL tokens on Solana. Powered by Anchor.**

---

## Project Overview

**Ticket Vault** is a production-ready staking protocol built with the **Anchor framework** on **Solana**. It enables users to stake:

- **Native SOL**
- **NFTs** (Non-Fungible Tokens)
- **SPL Tokens** (Fungible)

...and earn **time-based rewards** with two staking modes:

- **Flexible**: Unstake anytime
- **Locked**: Minimum **60 seconds** lock → **+50% reward boost**

All assets are secured using **Program-Derived Addresses (PDAs)** and **Token Program freezing** — ensuring **zero risk of unauthorized transfer** during staking.

---

## Key Features

| Feature | Description |
|--------|-------------|
| **Multi-Asset Support** | Stake SOL, NFTs, and any SPL token |
| **NFT Freeze/Thaw** | Lock NFTs via `freeze`, unlock with `thaw` |
| **PDA Escrow Vault** | All assets held in program-controlled accounts |
| **Flexible & Locked Modes** | Choose instant withdrawal or higher APY |
| **Reward Engine** | Pro-rata rewards based on amount & duration |
| **Anchor-Powered** | Type-safe, auditable, and easy to extend |

---

## NFT Freezing Mechanism

When an NFT is staked, it is **frozen** using the Solana Token Program to prevent transfers or metadata changes.

## Installation

**To install the necessary dependencies, run the following command:**

  ```bash
  yarn install
  ```
  or
  ```bash
  npm install
  ```

## Usage

**To build the project, use:**
  ```bash
  anchor build
  ```

**To run tests, use:**
  ```bash
  anchor test
  ```

**To deploy the project, use the following command:**
  ```bash
  anchor deploy
  ```

### Staking Modes

| Mode       | Lock Period         | Reward Rate       | Unstake Anytime? |
|------------|---------------------|-------------------|------------------|
| **Flexible** | No lock (0 sec)     | **Base APY**      | Yes             |
| **Locked**   | **≥ 60 seconds**    | **+50% Boosted APY** | No (until lock expires) |

> **To earn extra yield, assets must be locked for a minimum of 60 seconds.**  
> Flexible staking allows instant withdrawal but earns only the **base reward rate**.

---

### Minimum Staking Period

```rust
const MIN_STAKE_PERIOD: u64 = 60; // 60 seconds
