# Starknet Adapter

A CLI to verify Cairo 1 programs on Starknet.

## Setup
- Run `cargo install --path .` to build the project and install the CLI

Currently, only Linux platforms are supported.

### Verifying on Starknet Sepolia
By default, the CLI verifies locally. To verify on Starknet Sepolia, you need some additional setup
- Install `sncast` following this [guide](https://foundry-rs.github.io/starknet-foundry/getting-started/installation.html)
- Create and deploy a new Starknet account with the name `testnet-sepolia` following this [guide](https://foundry-rs.github.io/starknet-foundry/starknet/account.html) -> can be checked in `~/.starknet_accounts/starknet_open_zeppelin_accounts.json`
- Add testnet tokens to the account via the [faucet](https://starknet-faucet.vercel.app/)

## Usage
- The program must be a Cairo 1 program
### Verifying locally
```bash
starknet-adapter run <program-path> --layout recursive --proof_mode
```

### Verifying on Starknet Sepolia
```bash
starknet-adapter run <program-path> --layout recursive --proof_mode --verify_on_starknet
```

## How it works
- Run Cairo 1 program via `cairo1-run`
- Run the prover with the generated public and private input files to create a proof
- Parse the proof file and verify it with Integrity's verifier either locally or on the Starknet Sepolia network
