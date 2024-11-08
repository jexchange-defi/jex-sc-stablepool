// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                           36
// Async Callback:                       1
// Total number of exported functions:  39

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    jex_sc_stablepool
    (
        init => init
        upgrade => upgrade
        configurePlatformFeesReceiver => configure_platform_fees_receiver
        configureUnderlyingPriceSource => configure_underlying_price_source
        setSwapFee => set_swap_fee
        issueLpToken => issue_lp_token
        enableMintBurn => enable_mint_burn
        pause => pause
        resume => resume
        allowSc => allow_sc
        enableAccessControl => enable_access_control
        disableAccessControl => disable_access_control
        addLiquidity => add_liquidity
        removeLiquidity => remove_liquidity
        removeLiquidityOneToken => remove_liquidity_one_token
        swap => swap
        estimateAmountOut => estimate_amount_out
        estimateAmountIn => estimate_amount_in
        estimateAddLiquidity => estimate_add_liquidity
        estimateRemoveLiquidity => estimate_remove_liquidity
        estimateRemoveLiquidityOneToken => estimate_remove_liquidity_one_token
        getAnalyticsForLastEpochs => get_analytics_for_last_epochs
        getStatus => get_status
        getTokens => tokens
        getAmpFactor => amp_factor
        getFees => lp_fees
        getTradingVolume => trading_volume
        getLiquidityFee => liquidity_fee
        getPlatformFeesReceiver => platform_fees_receiver
        getSwapFee => swap_fee
        getVirtualPrice => get_virtual_price
        getReserves => reserves
        getLpTokenSupply => lp_token_supply
        getLptoken => lp_token
        getmultipliers => multipliers
        getNbTokens => nb_tokens
        getUnderlyingPriceSource => underlying_price_source
        isPaused => is_paused
    )
}

multiversx_sc_wasm_adapter::async_callback! { jex_sc_stablepool }
