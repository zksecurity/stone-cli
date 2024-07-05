#!/bin/bash

# Check if ~/.stone-prover directory exists, if not create it
if [ ! -d "$HOME/.stone-prover" ]; then
    mkdir -p "$HOME/.stone-prover"
fi

# Check if cpu_air_prover exists, if not copy it
if [ ! -f "$HOME/.stone-prover/cpu_air_prover" ]; then
    if ! cp ./cpu_air_prover "$HOME/.stone-prover/cpu_air_prover"; then
        echo "Failed to copy cpu_air_prover. Please check if the file exists in the current directory."
        exit 1
    fi
    echo "cpu_air_prover copied successfully."
else
    echo "cpu_air_prover already exists in ~/.stone-prover."
fi

# Check if cpu_air_verifier exists, if not copy it
if [ ! -f "$HOME/.stone-prover/cpu_air_verifier" ]; then
    if ! cp ./cpu_air_verifier "$HOME/.stone-prover/cpu_air_verifier"; then
        echo "Failed to copy cpu_air_verifier. Please check if the file exists in the current directory."
        exit 1
    fi
    echo "cpu_air_verifier copied successfully."
else
    echo "cpu_air_verifier already exists in ~/.stone-prover."
fi

# Check if cairo1-run binary exists, if not copy it
if [ ! -f "$HOME/.stone-prover/cairo1-run" ]; then
    if ! cp ./cairo1-run "$HOME/.stone-prover/cairo1-run"; then
        echo "Failed to copy cairo1-run binary. Please check if the file exists in the current directory."
        exit 1
    fi
    echo "cairo1-run binary copied successfully to ~/.stone-prover."
else
    echo "cairo1-run binary already exists in ~/.stone-prover."
fi


# Check if corelib directory exists, if not download it
if [ ! -d "$HOME/corelib" ]; then
    # Clone the Cairo repository with specific depth and branch, move corelib, and remove the repository
    if ! git clone --depth=1 -b v2.6.4 https://github.com/starkware-libs/cairo.git; then
        echo "Failed to clone the repository. Please check your internet connection and try again."
        exit 1
    fi

    if ! cp -r ./cairo/corelib "$HOME"; then
        echo "Failed to copy corelib directory. Please check if the directory exists in the current directory."
        exit 1
    fi

    # clean up
    if ! rm -rf cairo/; then
        echo "Failed to remove the repository. Please check your permissions and try again."
        exit 1
    fi
    echo "corelib directory copied successfully to ~/.stone-prover."
else
    echo "corelib directory already exists in ~/.stone-prover."
fi


# Build the current package
if ! cargo build --release; then
    echo "Failed to build the package. Please check the error messages above and try again."
    exit 1
fi


if ! cp ./target/release/starknet-adapter "$HOME/.stone-prover/starknet-adapter"; then
    echo "Failed to copy starknet-adapter binary. Please check if the file exists in the current directory."
    exit 1
fi
echo "starknet-adapter binary copied successfully to ~/.stone-prover."

# Add the binaries to the PATH
# The if statement checks if the PATH update is already present in the shell configuration file
if ! grep -q 'export PATH="$HOME/.stone-prover:$PATH"' "$HOME/.bashrc"; then
    echo 'export PATH="$HOME/.stone-prover:$PATH"' >> "$HOME/.bashrc"
    echo "Added ~/.stone-prover to PATH in .bashrc."
else
    echo "~/.stone-prover is already in PATH in .bashrc."
fi

if ! grep -q 'export PATH="$HOME/.stone-prover:$PATH"' "$HOME/.zshrc"; then
    echo 'export PATH="$HOME/.stone-prover:$PATH"' >> "$HOME/.zshrc"
    echo "Added ~/.stone-prover to PATH in .zshrc."
else
    echo "~/.stone-prover is already in PATH in .zshrc."
fi

# Source the updated shell configuration
if [ -n "$BASH_VERSION" ]; then
    source "$HOME/.bashrc"
elif [ -n "$ZSH_VERSION" ]; then
    source "$HOME/.zshrc"
fi

echo "PATH updated successfully."
