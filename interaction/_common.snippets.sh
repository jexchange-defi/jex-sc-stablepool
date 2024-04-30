##
# Info
##

echo "Proxy: ${PROXY}"
echo "SC address: ${SC_ADDRESS:-Not deployed}"

##
# Owner endpoints
##

configurePlatformFeesReceiver() {
    read -p "Receiver address: " RECEIVER_ADDRESS

    mxpy contract call ${SC_ADDRESS} --recall-nonce --keyfile=${KEYFILE} --gas-limit=10000000 \
        --function="configurePlatformFeesReceiver" \
        --arguments "${RECEIVER_ADDRESS}" \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

setSwapFee() {
    read -p "Swap fee (10000=1%, 300=0.03%): " SWAP_FEE

    mxpy contract call ${SC_ADDRESS} --recall-nonce --keyfile=${KEYFILE} --gas-limit=10000000 \
        --function="setSwapFee" \
        --arguments "${SWAP_FEE}" \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

issueLpToken() {
    read -p 'Display name: ' DISPLAY_NAME
    read -p 'Ticker: ' TICKER

    mxpy contract call ${SC_ADDRESS} --recall-nonce --keyfile=${KEYFILE} --gas-limit=75000000 \
        --function="issueLpToken" \
        --arguments "str:${DISPLAY_NAME}" "str:${TICKER}" \
        --value 50000000000000000 \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

enableMintBurn() {
    mxpy contract call ${SC_ADDRESS} --recall-nonce --keyfile=${KEYFILE} --gas-limit=75000000 \
        --function="enableMintBurn" \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

configureUnderlyingPriceSource() {
    read -p 'Token ID: ' TOKEN_ID
    read -p 'Source address: ' SOURCE_ADDRESS
    read -p 'Endpoint name: ' ENDPOINT_NAME

    mxpy contract call ${SC_ADDRESS} --recall-nonce --keyfile=${KEYFILE} --gas-limit=10000000 \
        --function="configureUnderlyingPriceSource" \
        --arguments "str:${TOKEN_ID}" "${SOURCE_ADDRESS}" "str:${ENDPOINT_NAME}" \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

pause() {
    mxpy contract call ${SC_ADDRESS} --recall-nonce --keyfile=${KEYFILE} --gas-limit=10000000 \
        --function="pause" \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

resume() {
    mxpy contract call ${SC_ADDRESS} --recall-nonce --keyfile=${KEYFILE} --gas-limit=10000000 \
        --function="resume" \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

##
# Public endpoints
##

addLiquidity() {
    USER_ADDRESS=$(mxpy wallet convert --infile ${1} --in-format pem --out-format address-bech32 | tail -1)

    read -p "Nb tokens: " NB_TOKENS

    PAYMENTS=""
    for i in $(seq 1 ${NB_TOKENS})
    do
        read -p "$i) Token: " TOKEN
        read -p "$i) Amount: " AMOUNT

        PAYMENTS="${PAYMENTS} str:${TOKEN} 0 ${AMOUNT}"
    done
    set -x

    mxpy contract call ${USER_ADDRESS} --recall-nonce --pem=${1} --gas-limit=20000000 \
        --function="MultiESDTNFTTransfer" \
        --arguments "${SC_ADDRESS}" "${NB_TOKENS}" \
            ${PAYMENTS} \
            "str:addLiquidity" \
            "1" \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

swap() {
    read -p "Token IN: " TOKEN_IN
    read -p "Amount IN: " AMOUNT_IN
    read -p "Token OUT: " TOKEN_OUT

    mxpy contract call ${SC_ADDRESS} --recall-nonce --pem=${1} --gas-limit=10000000 \
        --function="ESDTTransfer" \
        --arguments "str:${TOKEN_IN}" "${AMOUNT_IN}" \
            "str:swap" "str:${TOKEN_OUT}" "1" \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

##
# Views
##

estimateAmountOut() {
    read -p "Token IN: " TOKEN_IN
    read -p "Amount IN: " AMOUNT_IN
    read -p "Token OUT: " TOKEN_OUT

    mxpy contract query ${SC_ADDRESS} \
        --function "estimateAmountOut" \
        --arguments "str:${TOKEN_IN}" "${AMOUNT_IN}" "str:${TOKEN_OUT}" \
        --proxy=${PROXY} | jq .[].number
}

getStatus() {
    mxpy contract query ${SC_ADDRESS} --function "getStatus" --proxy=${PROXY} \
        | jq .[].hex
}

getVirtualPrice() {
    mxpy contract query ${SC_ADDRESS} --function "getVirtualPrice" --proxy=${PROXY} \
        | jq .[].number
}
