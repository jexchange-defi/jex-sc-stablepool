#!/bin/bash

BYTECODE=../output-docker/jex-sc-stablepool/jex-sc-stablepool.wasm
KEYFILE="../../wallets/deployer-shard1.json"
PROXY=https://gateway.multiversx.com
MVX_TOOLS_URL=https://tools.multiversx.com
GUARDIAN_ADDRESS=erd1cvt4665apw4wwmjervlncdnhags8hdpu3l9rpca8y4xtz6njmyfqxatz8n
PLAY_API_URL=https://play-api.multiversx.com
SC_ADDRESS=$(mxpy data load --key=address-mainnet-ush-usdc-usdt)
CHAIN=1
SCRIPT_DIR=$(dirname $0)
SDK_RUST_CONTRACT_BUILDER_TAG=v6.1.0
AMP_FACTOR=256
USH_ID=USH-111e09
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

    read -p "2FA code: " TWO_FA_CODE

    mxpy contract deploy --bytecode=${BYTECODE} --metadata-payable \
        --arguments "${AMP_FACTOR}" \
            "str:${USH_ID}" "1" \
            "str:${USDC_ID}" "1000000000000" \
            "str:${USDT_ID}" "1000000000000" \
        --keyfile=${KEYFILE} --gas-limit=75000000 --outfile="deploy-mainnet.interaction.json" \
        --proxy=${PROXY} --chain=${CHAIN} \
        --guardian "${GUARDIAN_ADDRESS}" \
        --guardian-service-url "${MVX_TOOLS_URL}/guardian" \
        --guardian-2fa-code "${TWO_FA_CODE}" \
        --version 2 --options 2 \
        --recall-nonce --send || return

    SC_ADDRESS=$(cat deploy-mainnet.interaction.json | jq -r .contractAddress)

    mxpy data store --key=address-mainnet-ush-usdc-usdt --value=${SC_ADDRESS}

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
