# Stone CLI

A CLI for proving and verifying Cairo programs.

## Overview

![Stone CLI structure](./assets/stone-cli-structure.svg)

As shown in the diagram, the lifecycle of a Cairo program is composed of mainly four steps: compile & run, prove, serialize, and verify. The Stone CLI provides commands for each of these steps, except for the compile & run step, which is merged into the prove step.

Once a proof has been generated, it can be verified on three different verifiers: [Stone verifier in C++](https://github.com/starkware-libs/stone-prover), [Starknet verifier in Cairo](https://github.com/HerodotusDev/integrity), and [Ethereum verifier in Solidity](https://github.com/zksecurity/stark-evm-adapter). The Stone verifier is a local verifier that can be used for testing purposes, while the Starknet and Ethereum verifiers are verifiers deployed on-chain that can be used to verify the execution of a Cairo program. Once the execution is verified, the input and the output of the program can be registered as a "fact" as on-chain data and can be used by any other smart contracts.

For example, say you want to have an on-chain proof that the 10th Fibonacci number is 55. You can run the Stone CLI on a Cairo program that implements the Fibonacci sequence, providing `10` as input. The CLI will create a proof along with some public information that includes the bytecode of your Cairo program and the inputs and outputs. Once you submit the proof to an on-chain verifier, the verifier will verify the proof and register the inputs and outputs as a "fact" that is associated with the hash of the bytecode of your Cairo program. As a result, any smart contract can make use of the fact that the 10th Fibonacci number is 55.

As can be seen in the diagram, the Stone CLI does not directly interact with the on-chain verifiers. Instead, it serializes the proof into a format that is compatible with the on-chain verifier, which can then be verified using the [Starknet verifier](https://github.com/HerodotusDev/integrity) or the [Ethereum verifier](https://github.com/zksecurity/stark-evm-adapter). Note that the proof is often split into multiple files since the entire proof usually does not fit inside the calldata limits of a single transaction.

For the Ethereum verifier, there is an additional specific requirement: the proof needs to be generated using a specific Cairo program named the "bootloader". The bootloader program allows one to efficiently run multiple Cairo programs by creating a smaller size proof (see more details in the [STARK book](https://zksecurity.github.io/stark-book/cairo/bootloader.html)). Since only the bootloader program is supported on Ethereum, the CLI provides an easy way to generate proofs using the bootloader program via the `prove-bootloader` command.

Please refer to the [Cairo book](https://book.cairo-lang.org/) for more details on how to create a Cairo program.

## Setup

Run the following command to install the CLI:

```bash
cargo install --path .
```

Currently, `linux/amd64` with `AVX` and `macos/arm64` are supported.

## Usage

### Prove

Generate a proof for a Cairo 0 or Cairo 1 program. This includes the process of compiling the program to CASM (Cairo Assembly) and running the CASM code to generate the memory and trace outputs. For Cairo 0 programs, the compile step needs to be done separately via the [`cairo-vm`](https://github.com/lambdaclass/cairo-vm?tab=readme-ov-file#running-cairo-vm-from-cli) CLI.

```bash
stone-cli prove --cairo_program <program-path>
```

Additional args:

- `--program_input`
- `--program_input_file`
- `--layout`: See [List of supported builtins per layout](#list-of-supported-builtins-per-layout)
- `--prover_config_file`
- `--parameter_file`
- `--output`
- `--stone_version`: [v5](https://github.com/starkware-libs/stone-prover/commit/7ac17c8ba63a789604350e501558ef0ab990fd88) and [v6](https://github.com/starkware-libs/stone-prover/commit/1414a545e4fb38a85391289abe91dd4467d268e1) are not compatible because v6 additionally [includes the `n_verifier_friendly_commitment_layers` value](https://github.com/starkware-libs/stone-prover/commit/1414a545e4fb38a85391289abe91dd4467d268e1#diff-ed7255be97fbeb539a95132b4f2dea9753b8a40f9f59ea220f3c2eeb3afd1fc1R94) when calculating the public input hash.

Additional args for prover parameters. Most of them are related to optimizations or the security level of the proof. You can refer to the [RFC](https://zksecurity.github.io/RFCs/) for more details on some of them.

- `--field`
- `--channel_hash`
- `--commitment_hash`
- `--n_verifier_friendly_commitment_layers`
- `--pow_hash`
- `--page_hash`
- `--fri_step_list`
- `--last_layer_degree_bound`
- `--n_queries`
- `--proof_of_work_bits`
- `--log_n_cosets`
- `--use_extension_field`
- `--verifier_friendly_channel_updates`
- `--verifier_friendly_commitment_hash`

Additional args for prover config:

- `--store_full_lde`
- `--use_fft_for_eval`
- `--constraint_polynomial_task_size`
- `--n_out_of_memory_merkle_layers`
- `--table_prover_n_tasks_per_segment`

### Prove bootloader

Generate a proof for the bootloader Cairo program

```bash
stone-cli prove-bootloader --cairo_program <program-path>
```

Additional args:

- `--program_input`
- `--program_input_file`
- `--layout`
- `--prover_config_file`
- `--parameter_file`
- `--ignore_fact_topologies`

### Verify

Verify a proof generated by the prover

```bash
stone-cli verify --proof <proof-path>
```

Additional args:

- `--annotation_file`
- `--extra_output_file`

`--annotation_file` and `--extra_output_file` arguments are required when serializing a proof for Ethereum.

### Serialize Proof

- Serialize a proof to be verified on Starknet or Ethereum
- Ethereum
  - `stone-cli serialize-proof --proof <proof-path> --network ethereum --annotation_file <annotation-path> --extra_output_file <extra-output-path> --output <output-path>`
- Starknet
  - [integrity](https://github.com/HerodotusDev/integrity) provides two types of serializations for Starknet
  - monolith type (supports only `recursive` layout)
    - `stone-cli serialize-proof --proof <proof-path> --network starknet --serialization_type monolith --output <output-path>`
  - split type (supports `dex`, `small`, `recursive`, `recursive_with_poseidon`, `starknet`, and `starknet_with_keccak` layouts)
    - `stone-cli serialize-proof --proof <proof-path> --network starknet --serialization_type split --output_dir <output-dir> --layout starknet`

### How to create proofs and verify them on Ethereum

![Proving and verifying on Ethereum](./assets/stone-cli-workflow2.svg)

Currently there is a Solidity verifier deployed on Ethereum, which is mainly used to verify SHARP proofs created by L2 Starknet nodes. The Solidity verifier checks the validity of a Cairo program named `bootloader`, which can prove the execution of multiple Cairo programs or Cairo PIEs (Position Independent Executable) either by executing them directly in the program or by running a Cairo verifier that recursively verifies (i.e. verify a proof inside the program) a bootloader proof. The bootloader program dramatically lowers the cost of verification as proving a new Cairo program will grow the size of the proof logarithmically as opposed to linearly. Once we create a bootloader proof, we need to serialize it to a format that works for the Cairo verifier on Ethereum. (Note: the Solidity verifier is based on Stone version `v5`, so the `--stone_version` argument needs to be set to `v5`)

Here are the specific steps for the above process:

1. Call `stone-cli prove-bootloader --cairo_programs ./examples/cairo0/bitwise_output.json --layout starknet --parameter_file ./tests/configs/bootloader_cpu_air_params.json --output bootloader_proof.json --fact_topologies_output fact_topologies.json`

   - Can also provide multiple programs and pies by providing a space-separated list of paths

2. Call `stone-cli verify --proof bootloader_proof.json --annotation_file annotation.json --extra_output_file extra_output.json --stone_version v5`

3. Call `stone-cli serialize-proof --proof bootloader_proof.json --annotation_file annotation.json --extra_output_file extra_output.json --network ethereum --output bootloader_serialized_proof.json`

4. Verify on Ethereum with the [evm-adapter CLI](https://github.com/zksecurity/stark-evm-adapter/tree/add-build-configs?tab=readme-ov-file#using-existing-proof) using the `bootloader_serialized_proof.json` and `fact_topologies.json` files as inputs

### How to create proofs and verify them on Starknet

![Proving and verifying on Starknet](./assets/stone-cli-workflow1.svg)

1. Call `stone-cli prove --cairo_program <program-path> --layout <layout>` with a layout that is supported by either the `monolith` or `split` serialization types

2. Call `stone-cli serialize-proof --proof <proof-path> --network starknet --serialization_type monolith --output <output-path>` or `stone-cli serialize-proof --proof <proof-path> --network starknet --serialization_type split --output_dir <output-dir> --layout <layout>`

3. Verify on Starknet with [integrity](https://github.com/HerodotusDev/integrity) using the `output` file or files in the `output_dir` as input

#### Notes

- Cairo 0 programs that use hints are not supported
- Only the `starknet` layout is supported for bootloader proofs
- Programs should use the `output` builtin--programs that do not can be proved, but won't verify on Ethereum

## Testing

Before running the tests, make sure to increase the Rust default stack size via `export RUST_MIN_STACK=4194304`. After that, you can run `cargo test` to run all the tests.

## Versioning guide

- Minor version changes should be made when the underlying `cairo1-run` binary built from [cairo-vm](https://github.com/lambdaclass/cairo-vm) is updated.
- When updating the `cairo1-run` binary, the `cairo` release version specified in `build.rs` should also be updated to a compatible version.

## Additional Resources

### List of supported builtins per layout

|             | small | recursive | dex | recursive_with_poseidon | starknet | starknet_with_keccak |
| ----------- | :---: | :-------: | :-: | :---------------------: | :------: | :------------------: |
| output      |   O   |     O     |  O  |            O            |    O     |          O           |
| pedersen    |   O   |     O     |  O  |            O            |    O     |          O           |
| range_check |   O   |     O     |  O  |            O            |    O     |          O           |
| bitwise     |       |     O     |     |            O            |    O     |          O           |
| ecdsa       |       |           |  O  |                         |    O     |          O           |
| poseidon    |       |           |     |            O            |    O     |          O           |
| ec_op       |       |           |     |                         |    O     |          O           |
| keccak      |       |           |     |                         |          |          O           |

## Common issues

```bash
Error: Failed to run cairo1: cairo1-run failed with error: Error: VirtualMachine(Memory(AddressNotRelocatable))
```

This error occurs when the program uses a builtin that is not supported by the layout. Refer to the [List of supported builtins per layout](#list-of-supported-builtins-per-layout) to find the right layout for theprogram.

```bash
thread 'opt cgu.00' has overflowed its stack
fatal runtime error: stack overflow
error: could not compile `swiftness_air` (lib)
```

This error occurs when trying to run `cargo test`. This can be solved by increasing the Rust default stack size via `export RUST_MIN_STACK=4194304`.
