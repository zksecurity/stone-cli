# Starknet Adapter

A CLI to run Cairo 1 programs on Starknet.

## Setup
- Run `./install-dependencies.sh` to install all dependencies.
- Setup `sncast` following this [guide](https://foundry-rs.github.io/starknet-foundry/getting-started/installation.html)
- Create and deploy a new Starknet account with the name `testnet-sepolia` following this [guide](https://foundry-rs.github.io/starknet-foundry/starknet/account.html) -> can be checked in `~/.starknet_accounts/starknet_open_zeppelin_accounts.json`
- Add testnet tokens to the account via the [faucet](https://starknet-faucet.vercel.app/)

## Usage

```bash
starknet-adapter run <program-path> --layout recursive --proof_mode
```
- The program must be a Cairo 1 program


## How it works
- Run Cairo 1 program via `cairo1-run`
- Use the generated public and private input files to run prover
- Parse the proof file and verify it on the Starknet Sepolia network using `sncast`
