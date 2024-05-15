# Stable swap smart contract (by JEXchangeDefi)

An implementation of stable swap AMM smart contract from "Solidity by example" on MultiversX.

Source: https://solidity-by-example.org/defi/stable-swap-amm/ (MIT licence)

This smart contract also implements concepts from Curve "lending" pools, where asset theorical prices are delegated to another smart contract (liquid staking protocol, oracle, ...).

# License

The source code is distributed under [MIT license](LICENSE).

Please make sure to read it before using this smart contract.

# Step-by-step guide

## Build the smart contract

First, build the smart contract using `mxpy` (see [here](https://docs.multiversx.com/sdk-and-tools/sdk-py/mxpy-cli/#building-a-smart-contract]))

## Deploy the smart contract

Deploy the smart contract using `mxpy` (see [here](https://docs.multiversx.com/sdk-and-tools/sdk-py/mxpy-cli/#deploying-a-smart-contract)).

Params:

- `amp_factor`: curve amplification factor (see [here](#references))
- `tokens_and_multipliers`: list of token identifiers + multipliers. Multipliers are used to normalize token amounts (ie. manage difference of decimals between tokens)

Example:

- `amp_factor` = 256
- `tokens_and_multipliers` =
  - "WDAI-9eeb54"
  - 1
  - "USDC-c76f1f"
  - 10^12
  - "USDT-f8c08c"
  - 10^12

Use a gas limit of `75_000_000` units.

## Create LP token

### Issue LP token

Endpoint: `issueLpToken`

Payment: `0.5 EGLD` (blockchain fee to issue an ESDT)

Params:

- `lp_token_display_name`: LP token display name (max 20 characters)
- `lp_token_ticker`: LP token ticker (max 10 characters)

### Enable LP token mint & burn

Endpoint: `enableMintBurn`

Params: None

## Configure

### Set platform fees receiver

33% of swap fees are considered as platform fees. They are sent to a dedicated wallet.

Endpoint: `configurePlatformFeesReceiver`

Params:

- receiver: fees receiver wallet address

### Set swap fee

Configure swap fee.

Note that 33% of swap fees are sent to the platform fees wallet; the rest remains in the pool, providing yield for liquidity providers.

Endpoint: `setSwapFee`

Params:

- swap_fee: swap fee in percent base point (eg: `10000` = 1%, `300` = 0.03%)

## Configure underlying price sources

Assets are not also designed to remain pegged 1:1 to one another. Instead, the theorical ratio between assets depends on an external source (liquid staking protocol, oracle, ...).

The external source must be queryable on-chain through a smart contract view with no parameters.

Considering `A` and `B` tokens, if `B = x * A`, then the smart contract view must return a ratio such that:

`price(A) * ratio / 10^18 = price(B)`

Note that the external smart contract must be deployed on the same shard as your stable swap smart contract.

Endpoint: `configureUnderlyingPriceSource`

Params:

- `token`: token identifier
- `address`: smart contract address to query
- `endpoint_name`: name of the view to query

## Pause & Resume

The smart contract has a security mechanism that can be used in case of emergency (for example, if a depeg of 1 of the assets occurs).

Pausing a smart contract prevents users from swapping and depositing funds, **but does not prevent them from withdrawing their tokens.**

Note that by default, smart contract is paused. You need to [resume](#resume) it once configuration is complete.

### Pause

Endpoint: `pause`

Params: None

### Resume

Endpoints: `resume`

Params: None

## Liquidity management

### Deposit liquidity

Endpoint: `addLiquidity`

Payment: 1 or more ESDT payments

Params:

- `min_shares`: minimum amount of shares to receive (see [here](#views))

### Withdraw liquidity (all tokens)

Endpoint: `removeLiquidity`

Payment: shares (in LP tokens)

Params:

- `min_amounts`: minimum amounts to receive for each token

### Withdraw liquidity (1 token)

It is possible to withdraw liquidity in one token. Note that imbalance fee may apply.

Endpoint: `removeLiquidityOneToken`

Payment: shares (in LP tokens)

Params:

- `token_out`: token to receive
- `min_amount_out`: minimum amount to receive

## Swap

Endpoint: `swap`

Payment: 1 ESDT payment

Params:

- `token_out`: token to receive
- `min_amount_out`: minimum amount to receive

## Views

- estimateAmountOut
- estimateAddLiquidity
- estimateRemoveLiquidity
- estimateRemoveLiquidityOneToken
- getStatus

# References

- https://theammbook.org/formulas/stableswap/
- https://miguelmota.com/blog/understanding-stableswap-curve/

# Donate

Feel free to tip me by sending funds to

erd1ssruj9rjy529ajqpqfmtkyq422fh2m4zhkp4pmfng3aad2h7ua2quydm30 (MultiversX)
