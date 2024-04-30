#!/bin/bash

BYTECODE=../output-docker/jex-sc-stablepool/jex-sc-stablepool.wasm
KEYFILE="../../wallets/deployer-shard1.json"
PROXY=https://gateway.multiversx.com
SC_ADDRESS=$(mxpy data load --key=address-mainnet-segld-egld)
CHAIN=1
SCRIPT_DIR=$(dirname $0)
SDK_RUST_CONTRACT_BUILDER_TAG=v6.1.1
AMP_FACTOR=256
SEGLD_ID=SEGLD-3ad2d0
WEGLD_ID=WEGLD-bd4d79

source "${SCRIPT_DIR}/_common.snippets.sh"

build() {
    pushd ..
    rm -rf output-docker
    mxpy contract reproducible-build --docker-image="multiversx/sdk-rust-contract-builder:${SDK_RUST_CONTRACT_BUILDER_TAG}"
    popd
}

deploy() {
    echo 'You are about to deploy SC on mainnet (Ctrl-C to abort)'
    read answer

    mxpy contract deploy --bytecode=${BYTECODE} --metadata-payable \
        --arguments "${AMP_FACTOR}" \
            "str:${SEGLD_ID}" "1" \
            "str:${WEGLD_ID}" "1" \
        --keyfile=${KEYFILE} --gas-limit=75000000 --outfile="deploy-mainnet.interaction.json" \
        --proxy=${PROXY} --chain=${CHAIN} --recall-nonce --send || return

    SC_ADDRESS=$(cat deploy-mainnet.interaction.json | jq -r .contractAddress)

    mxpy data store --key=address-mainnet-segld-egld --value=${SC_ADDRESS}

    echo ""
    echo "Smart contract address: ${SC_ADDRESS}"
}

upgrade() {
    echo 'You are about to upgrade current SC on mainnet (Ctrl-C to abort)'
    read answer

    mxpy contract upgrade --bytecode=${BYTECODE} --metadata-payable \
        --keyfile=${KEYFILE} --gas-limit=75000000 --outfile="deploy-mainnet.interaction.json" \
        --proxy=${PROXY} --chain=${CHAIN} --recall-nonce --send ${SC_ADDRESS} || return

    echo ""
    echo "Smart contract upgraded: ${SC_ADDRESS}"
}


CMD=$1
shift

$CMD $*
