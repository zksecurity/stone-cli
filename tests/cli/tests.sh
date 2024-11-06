# Integration tests for the CLI
# tests against
# 1. https://github.com/zksecurity/stark-evm-adapter.git
# 2. https://github.com/HerodotusDev/integrity.git
# 3. https://github.com/zksecurity/integrity-calldata-generator.git -> forked because the CLI implementation also relies on the fork

#!/bin/bash
set -e

# Set main directory
MAIN_DIR=$(pwd)

# Install stone-cli
cargo install --path .

# Create and verify bootloader proof on Ethereum
echo "Creating and verifying bootloader proof on Ethereum"

if [ "$(uname)" = "Linux" ]; then
    stone-cli prove-bootloader \
        --cairo_programs examples/cairo0/array_sum.json \
        --parameter_file ./tests/configs/bootloader_cpu_air_params.json \
        --output bootloader_proof.json
    echo "Proof generated"
fi

if [ "$(uname)" = "Darwin" ]; then
    cp ./tests/cli/resources/macos-testing-proofs/bootloader_proof.json bootloader_proof.json
    cp ./tests/cli/resources/macos-testing-proofs/fact_topologies.json fact_topologies.json
    echo "Proof copied"
fi

stone-cli verify \
    --proof bootloader_proof.json \
    --annotation_file annotation.json \
    --extra_output_file extra_output.json \
    --stone_version v5
echo "Proof verified"

stone-cli serialize-proof \
    --proof bootloader_proof.json \
    --annotation_file annotation.json \
    --extra_output_file extra_output.json \
    --network ethereum \
    --output bootloader_serialized_proof.json
echo "Proof serialized"

git clone https://github.com/zksecurity/stark-evm-adapter.git
cd $MAIN_DIR/stark-evm-adapter/
URL=https://rpc.tenderly.co/fork/f4839248-30b4-4451-b1da-93ebb124c73f \
ANNOTATED_PROOF=../bootloader_serialized_proof.json \
FACT_TOPOLOGIES=../fact_topologies.json \
cargo run --example verify_stone_proof

echo "Proof verified on Ethereum"

# Create and verify proof on Starknet
echo "Creating and verifying proof on Starknet"
cd $MAIN_DIR

if [ "$(uname)" = "Linux" ]; then
    stone-cli prove \
        --cairo_program tests/resources/integrity-programs/cairo0_fibonacci_recursive_builtins.json \
        --cairo_version cairo0 \
        --layout recursive \
        --output cairo0_fibonacci_recursive_builtins_stone5_keccak_160_lsb.json \
        --stone_version v5
    echo "Proof generated"
fi

if [ "$(uname)" = "Darwin" ]; then
    cp ./tests/cli/resources/macos-testing-proofs/cairo0_fibonacci_recursive_builtins_stone5_keccak_160_lsb.json cairo0_fibonacci_recursive_builtins_stone5_keccak_160_lsb.json
    echo "Proof copied"
fi

# Serialize proof
stone-cli serialize-proof \
    --proof cairo0_fibonacci_recursive_builtins_stone5_keccak_160_lsb.json \
    --network starknet \
    --serialization_type monolith \
    --output cairo0_fibonacci_recursive_builtins_stone5_keccak_160_lsb
echo "Monolith proof serialized"

# Clone integrity repo
if [ ! -d "integrity" ]; then
    git clone https://github.com/HerodotusDev/integrity.git
    echo "Integrity repo cloned"
fi
cd $MAIN_DIR/integrity/

# Install sncast if it doesn't exist
if ! command -v sncast &> /dev/null; then
    echo "sncast not found, installing snfoundryup..."
    curl -L https://raw.githubusercontent.com/foundry-rs/starknet-foundry/master/scripts/install.sh | sh
    export PATH="/root/.local/bin:$PATH"
    source $HOME/.bashrc
    snfoundryup
    sncast --version
    echo "sncast installed"
fi

# Create starknet accounts directory and file if they don't exist
mkdir -p ~/.starknet_accounts

# Write account config to file (monolith proof uses `my-sepolia-account` and split proof uses `my_account`)
cat > ~/.starknet_accounts/starknet_open_zeppelin_accounts.json << 'EOL'
{
  "alpha-sepolia": {
    "my-sepolia-account": {
      "address": "0x5044ea2b88229bb5a1ff5424ee746ed1a12541b9a1ec14e767d02c67ddb1fc1",
      "class_hash": "0xe2eb8f5672af4e6a4e8a8f1b44989685e668489b0a25437733756c5a34a1d6",
      "deployed": true,
      "legacy": false,
      "private_key": "0x713504d58868bbbf4b6447842b2e797f1a344a364a5298b07e3b9e93eab06dc",
      "public_key": "0x144a636155881974dcc21193786db7d879c6edf4646a17d6a5559034ef7a802",
      "salt": "0xd1531c0bb45f4634",
      "type": "open_zeppelin"
    },
    "my_account": {
      "address": "0x5dd7aa049cfd03cc0d95ec6d562dda8321f8681e4a9635c57aefa738c8a3661",
      "class_hash": "0xe2eb8f5672af4e6a4e8a8f1b44989685e668489b0a25437733756c5a34a1d6",
      "deployed": true,
      "legacy": false,
      "private_key": "0x5908f00352439ab34724e7d50681fe2303c03ac52d0c2f02e2ac254e05bcd8a",
      "public_key": "0x7aa593b1fd343fc8079feee02cd253a10b6efeca294b689d94c474a3a6907b9",
      "salt": "0x479a8932e6bc52ad",
      "type": "open_zeppelin"
    }
  }
}
EOL
echo "Starknet accounts file created"

./verify-on-starknet.sh \
    0x16409cfef9b6c3e6002133b61c59d09484594b37b8e4daef7dcba5495a0ef1a \
    ../cairo0_fibonacci_recursive_builtins_stone5_keccak_160_lsb \
    recursive \
    keccak_160_lsb \
    stone5 \
    cairo0
echo "Monolith proof verified on Starknet"

cd $MAIN_DIR
stone-cli serialize-proof \
    --proof cairo0_fibonacci_recursive_builtins_stone5_keccak_160_lsb.json \
    --network starknet \
    --serialization_type split \
    --output_dir split_proofs \
    --layout recursive
echo "Split proof serialized"

# Verify split proofs using the integrity-calldata-generator repo
cd $MAIN_DIR
if [ ! -d "integrity-calldata-generator" ]; then
    GIT_LFS_SKIP_SMUDGE=1 git clone https://github.com/zksecurity/integrity-calldata-generator
fi
echo "integrity-calldata-generator repo cloned"

# Copy split proofs to integrity-calldata-generator
cp -r $MAIN_DIR/split_proofs/* $MAIN_DIR/integrity-calldata-generator/cli/calldata/

# Generate random job id
if [ "$(uname)" = "Linux" ]; then
    JOB_ID=$(shuf -i 1-10000000000000 -n 1)
elif [ "$(uname)" = "Darwin" ]; then
    JOB_ID=$(jot -r 1 1 10000000000000)
fi
echo "Generated random job id: $JOB_ID"

# Run verify.sh
cd $MAIN_DIR/integrity-calldata-generator/cli
./verify.sh $JOB_ID recursive keccak_160_lsb stone5 cairo0

echo "Split proof verified on Starknet"
