#!/bin/bash

BYTECODE=../output/jex-sc-stablepool.wasm
KEYFILE="../../wallets/deployer-shard1.json"
PROXY=https://devnet-gateway.multiversx.com
SC_ADDRESS=$(mxpy data load --key=address-devnet)
CHAIN=D
SCRIPT_DIR=$(dirname $0)
AMP_FACTOR=256
BUSD_ID=BUSD-632f7d
USDC_ID=USDC-8d4068
USDT_ID=USDT-188935

source "${SCRIPT_DIR}/_common.snippets.sh"

deploy() {
    echo 'You are about to deploy SC on devnet (Ctrl-C to abort)'
    read answer

    mxpy contract deploy --bytecode=${BYTECODE} --metadata-payable \
        --arguments "${AMP_FACTOR}" \
            "str:${BUSD_ID}" "1" \
            "str:${USDC_ID}" "1000000000000" \
            "str:${USDT_ID}" "1000000000000" \
        --keyfile=${KEYFILE} --gas-limit=75000000 --outfile="deploy-devnet.interaction.json" \
        --proxy=${PROXY} --chain=${CHAIN} --recall-nonce --send || return

    SC_ADDRESS=$(mxpy data parse --file="deploy-devnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-devnet --value=${SC_ADDRESS}

    echo ""
    echo "Smart contract address: ${SC_ADDRESS}"
}

upgrade() {
    echo 'You are about to upgrade current SC on devnet (Ctrl-C to abort)'
    read answer

    mxpy contract upgrade --bytecode=${BYTECODE} --metadata-payable \
        --arguments "0x" \
            "0x" "0x" \
        --keyfile=${KEYFILE} --gas-limit=75000000 --outfile="deploy-devnet.interaction.json" \
        --proxy=${PROXY} --chain=${CHAIN} --recall-nonce --send ${SC_ADDRESS} || return

    echo ""
    echo "Smart contract upgraded: ${SC_ADDRESS}"
}

CMD=$1
shift

$CMD $*
