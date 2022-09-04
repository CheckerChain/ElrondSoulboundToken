PEM_FILE="./soulbound.pem"
SOULBOUND_CONTRACT="output/soulbound.wasm"

PROXY_ARGUMENT="--proxy=https://devnet-api.elrond.com"
CHAIN_ARGUMENT="--chain=D"

build_soulbound() {
    (set -x; erdpy --verbose contract build "$SOULBOUND_CONTRACT")
}

deploy_soulbound() {
    local TOKEN_NAME=0x45474c44 # "EGLD"
    local TOKEN_NAME=0x45474c44 # "EGLD"

    
    local OUTFILE="out.json"
    (set -x; erdpy contract deploy --bytecode="$SOULBOUND_CONTRACT" \
        --pem="$PEM_FILE" \
        $PROXY_ARGUMENT $CHAIN_ARGUMENT \
        --outfile="$OUTFILE" --recall-nonce --gas-limit=60000000 \
        --arguments ${TOKEN_NAME} ${TOKEN_NAME} ${DURATION} --send \
        || return)

    local RESULT_ADDRESS=$(erdpy data parse --file="$OUTFILE" --expression="data['emitted_tx']['address']")
    local RESULT_TRANSACTION=$(erdpy data parse --file="$OUTFILE" --expression="data['emitted_tx']['hash']")

    echo ""
    echo "Deployed contract with:"
    echo "  \$RESULT_ADDRESS == ${RESULT_ADDRESS}"
    echo "  \$RESULT_TRANSACTION == ${RESULT_TRANSACTION}"
    echo ""
}
