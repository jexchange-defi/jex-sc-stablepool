#!/bin/bash

BYTECODE=../output-docker/jex-sc-stablepool/jex-sc-stablepool.wasm
KEYFILE="../../wallets/deployer-shard1.json"
PROXY=https://gateway.multiversx.com
SC_ADDRESS=$(mxpy data load --key=address-mainnet-usdc-usdt)
CHAIN=1
SCRIPT_DIR=$(dirname $0)
SDK_RUST_CONTRACT_BUILDER_TAG=v5.3.0
AMP_FACTOR=256
USDC_ID=USDC-c76f1f
USDT_ID=USDT-f8c08c

source "${SCRIPT_DIR}/_common.snippets.sh"

build() {
    pushd ..
    mxpy contract reproducible-build --docker-image="multiversx/sdk-rust-contract-builder:${SDK_RUST_CONTRACT_BUILDER_TAG}"
    popd
}

deploy() {
    echo 'You are about to deploy SC on mainnet (Ctrl-C to abort)'
    read answer

    mxpy contract deploy --bytecode=${BYTECODE} --metadata-payable \
        --arguments "${AMP_FACTOR}" \
            "str:${USDC_ID}" "1000000000000" \
            "str:${USDT_ID}" "1000000000000" \
        --keyfile=${KEYFILE} --gas-limit=75000000 --outfile="deploy-mainnet.interaction.json" \
        --proxy=${PROXY} --chain=${CHAIN} --recall-nonce --send || return

    SC_ADDRESS=$(mxpy data parse --file="deploy-mainnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-mainnet-usdc-usdt --value=${SC_ADDRESS}

    echo ""
    echo "Smart contract address: ${SC_ADDRESS}"
}

upgrade() {
    echo 'You are about to upgrade current SC on mainnet (Ctrl-C to abort)'
    read answer

    mxpy contract upgrade --bytecode=${BYTECODE} --metadata-payable --metadata-not-upgradeable \
        --arguments "0x" \
            "0x" "0x" \
        --keyfile=${KEYFILE} --gas-limit=75000000 --outfile="deploy-mainnet.interaction.json" \
        --proxy=${PROXY} --chain=${CHAIN} --recall-nonce --send ${SC_ADDRESS} || return

    echo ""
    echo "Smart contract upgraded: ${SC_ADDRESS}"
}

CMD=$1
shift

$CMD $*
