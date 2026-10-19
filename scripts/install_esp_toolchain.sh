#!/usr/bin/env bash
# Automated ESP-RS Toolchain Installer for Rusty_Allstar
set -euo pipefail

echo "============================================================"
echo " Rusty_Allstar: ESP32 Toolchain Installation Script"
echo "============================================================"

# Check for cargo
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Cargo / Rust is not installed or not in PATH."
    exit 1
fi

echo "--> Step 1/3: Installing espup (Espressif toolchain installer)..."
if ! command -v espup &> /dev/null; then
    cargo install espup --locked
else
    echo "espup is already installed."
fi

echo "--> Step 2/3: Installing espflash & ldproxy..."
if ! command -v espflash &> /dev/null; then
    cargo install espflash --locked
else
    echo "espflash is already installed."
fi

if ! command -v ldproxy &> /dev/null; then
    cargo install ldproxy --locked
else
    echo "ldproxy is already installed."
fi

echo "--> Step 3/3: Running espup to install Xtensa Rust toolchain & LLVM..."
espup install

echo ""
echo "============================================================"
echo " Installation Complete!"
echo " Please run the following command or add it to your shell rc:"
echo "   source \$HOME/export-esp.sh"
echo "============================================================"
