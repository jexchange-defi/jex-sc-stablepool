#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

mod amm;
mod analytics;
mod fees;
mod liquidity;
mod maths;
mod pausable;

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct PairStatus<M: ManagedTypeApi> {
    paused: bool,
    amp_factor: u32,
    nb_tokens: usize,
    tokens: ManagedVec<M, TokenIdentifier<M>>,
    reserves: ManagedVec<M, BigUint<M>>,
    lp_token_identifier: TokenIdentifier<M>,
    lp_token_supply: BigUint<M>,
    owner: ManagedAddress<M>,
    swap_fee: u32,
    platform_fees_receiver: Option<ManagedAddress<M>>,
    volume_prev_epoch: ManagedVec<M, BigUint<M>>,
    fees_prev_epoch: ManagedVec<M, BigUint<M>>,
    fees_last_7_epochs: ManagedVec<M, BigUint<M>>,
}

#[multiversx_sc::contract]
pub trait JexScStablepoolContract:
    amm::AmmModule
    + analytics::AnalyticsModule
    + fees::FeesModule
    + liquidity::LiquidityModule
    + maths::MathsModule
    + pausable::PausableModule
{
    /// Smart contract init.
    ///
    /// tokens_and_multipliers: list of tuples (token identifier, multiplier).
    /// multipliers are used to convert each balance so it can be compared to others
    /// eg. BUSD has 18 decimals -> multiplier = 1, USDC has 6 decimals -> multiplier = 1e12, etc...
    #[init]
    fn init(
        &self,
        amp_factor: u32,
        tokens_and_multipliers: MultiValueEncoded<MultiValue2<TokenIdentifier, BigUint>>,
    ) {
        require!(tokens_and_multipliers.len() > 1, "Invalid number of tokens");

        require!(amp_factor > 0, "Invalid amp factor");

        self.amp_factor().set(amp_factor);

        self.nb_tokens().set(tokens_and_multipliers.len());

        let mut i = 0usize;
        for multi_value in tokens_and_multipliers {
            let (token, multiplier) = multi_value.into_tuple();
            self.tokens(i).set(&token);
            self.multipliers(i).set(&multiplier);

            // check token has not been used already
            for j in 0usize..i {
                require!(self.tokens(j).get() != token, "Tokens must be different");
            }

            i += 1;
        }

        self.do_pause();
    }

    #[upgrade]
    fn upgrade(&self) {}

    //
    // owner endpoints
    //

    #[only_owner]
    #[endpoint(configurePlatformFeesReceiver)]
    fn configure_platform_fees_receiver(&self, receiver: &ManagedAddress) {
        self.do_configure_platform_fees_receiver(receiver);
    }

    #[only_owner]
    #[endpoint(setSwapFee)]
    fn set_swap_fee(&self, swap_fee: u32) {
        self.update_swap_fee(self.nb_tokens().get(), swap_fee);
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueLpToken)]
    fn issue_lp_token(&self, lp_token_display_name: ManagedBuffer, lp_token_ticker: ManagedBuffer) {
        require!(self.lp_token().is_empty(), "LP token already issued");

        let egld_value = self.call_value().egld_value().clone_value();
        let caller = self.blockchain().get_caller();

        self.send()
            .esdt_system_sc_proxy()
            .issue_fungible(
                egld_value,
                &lp_token_display_name,
                &lp_token_ticker,
                &BigUint::from(1_000u32),
                FungibleTokenProperties {
                    num_decimals: 18,
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_mint: true,
                    can_burn: true,
                    can_change_owner: true,
                    can_upgrade: true,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().lp_token_issue_callback(&caller))
            .call_and_exit();
    }

    #[only_owner]
    #[endpoint(enableMintBurn)]
    fn enable_mint_burn(&self) {
        let lp_token = self.lp_token().get();
        require!(lp_token.is_valid_esdt_identifier(), "LP token not issued");

        let roles = [EsdtLocalRole::Mint, EsdtLocalRole::Burn];

        let sc_address = self.blockchain().get_sc_address();

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(&sc_address, &lp_token, roles.iter().cloned())
            .async_call()
            .call_and_exit();
    }

    #[only_owner]
    #[endpoint]
    fn pause(&self) {
        self.do_pause();
    }

    #[only_owner]
    #[endpoint]
    fn resume(&self) {
        self.do_unpause();
    }

    //
    // Public endpoints
    //

    #[endpoint(addLiquidity)]
    #[payable("*")]
    fn add_liquidity(&self, min_shares: BigUint) -> BigUint {
        self.require_not_paused();

        let payments = self.call_value().all_esdt_transfers();
        require!(payments.len() > 0, "No payment");

        let mut amounts = ManagedVec::<Self::Api, BigUint>::new();
        let mut nb_valid_payments = 0usize;

        for i in 0..self.nb_tokens().get() {
            let token = self.tokens(i).get();
            let mut amount = BigUint::zero();
            for payment in payments.iter() {
                if payment.token_identifier == token {
                    amount += payment.amount;
                    nb_valid_payments += 1;
                }
            }
            amounts.push(amount);
        }

        require!(nb_valid_payments == payments.len(), "Invalid payment token");

        let shares = self.do_add_liquidity(amounts, false);

        require!(shares >= min_shares, "Max slippage exceeded");

        self.send().direct_esdt(
            &self.blockchain().get_caller(),
            &self.lp_token().get(),
            0u64,
            &shares,
        );

        shares
    }

    #[endpoint(removeLiquidity)]
    #[payable("*")]
    fn remove_liquidity(
        &self,
        min_amounts: MultiValueEncoded<BigUint>,
    ) -> MultiValueEncoded<EsdtTokenPayment> {
        let (token_in, amount_in) = self.call_value().single_fungible_esdt();

        require!(token_in == self.lp_token().get(), "Invalid payment token");

        let min_amounts_vec = min_amounts.to_vec();
        let amounts_out = self.do_remove_liquidity(&amount_in, false);

        let mut payments_out = ManagedVec::<Self::Api, EsdtTokenPayment>::new();
        let mut i = 0usize;
        for amount_out in amounts_out.into_iter() {
            let min_amount = min_amounts_vec.get(i).clone_value();
            require!(&amount_out >= &min_amount, "Max slippage exceeded");

            payments_out.push(EsdtTokenPayment::new(
                self.tokens(i).get(),
                0u64,
                amount_out.clone(),
            ));

            i += 1;
        }

        self.send()
            .direct_multi(&self.blockchain().get_caller(), &payments_out);

        payments_out.into()
    }

    #[endpoint(removeLiquidityOneToken)]
    #[payable("*")]
    fn remove_liquidity_one_token(
        &self,
        token_out: TokenIdentifier,
        min_amount_out: BigUint,
    ) -> EsdtTokenPayment {
        let (token_in, amount_in) = self.call_value().single_fungible_esdt();

        require!(token_in == self.lp_token().get(), "Invalid payment token");

        let index_token_out = self.get_token_index(&token_out);

        let amount_out = self.do_remove_liquidity_one_token(amount_in, index_token_out, false);

        require!(amount_out >= min_amount_out, "Max slippage exceeded");

        let payment_out = EsdtTokenPayment::new(token_out, 0u64, amount_out);

        self.send().direct_esdt(
            &self.blockchain().get_caller(),
            &payment_out.token_identifier,
            payment_out.token_nonce,
            &payment_out.amount,
        );
        payment_out
    }

    #[endpoint(swap)]
    #[payable("*")]
    fn swap(&self, token_out: TokenIdentifier, min_amount_out: BigUint) -> EsdtTokenPayment {
        require!(min_amount_out > 1, "Invalid min amount to receive");

        self.require_not_paused();

        let (token_in, amount_in) = self.call_value().single_fungible_esdt();

        let index_token_in = self.get_token_index(&token_in);
        let index_token_out = self.get_token_index(&token_out);

        let (amount_out, lp_fee, platform_fee) =
            self.do_swap(index_token_in, index_token_out, amount_in.clone(), false);

        require!(amount_out >= min_amount_out, "Max slippage exceeded");

        let payment_out = EsdtTokenPayment::new(token_out, 0u64, amount_out);

        self.send().direct_esdt(
            &self.blockchain().get_caller(),
            &payment_out.token_identifier,
            payment_out.token_nonce,
            &payment_out.amount,
        );

        if &platform_fee > &0 {
            self.send().direct_esdt(
                &self.platform_fees_receiver().get(),
                &payment_out.token_identifier,
                payment_out.token_nonce,
                &platform_fee,
            );
        }

        self.analytics_add_lp_fees(&payment_out.token_identifier, &lp_fee);
        self.analytics_add_volume(&token_in, &amount_in);
        self.analytics_add_volume(&payment_out.token_identifier, &payment_out.amount);

        payment_out
    }

    //
    // Views
    //

    #[view(estimateAmountOut)]
    fn estimate_amount_out(
        &self,
        token_in: TokenIdentifier,
        amount_in: BigUint,
        token_out: TokenIdentifier,
    ) -> BigUint {
        self.require_not_paused();

        let index_token_in = self.get_token_index(&token_in);
        let index_token_out = self.get_token_index(&token_out);

        let (amount_out, _, _) = self.do_swap(index_token_in, index_token_out, amount_in, true);

        amount_out
    }

    #[view(estimateAddLiquidity)]
    fn estimate_add_liquidity(&self, amounts: MultiValueEncoded<BigUint>) -> BigUint {
        self.require_not_paused();

        let shares = self.do_add_liquidity(amounts.to_vec(), true);

        shares
    }

    #[view(estimateRemoveLiquidity)]
    fn estimate_remove_liquidity(&self, shares: BigUint) -> MultiValueEncoded<BigUint> {
        let amounts_out = self.do_remove_liquidity(&shares, true);

        amounts_out.into()
    }

    #[view(estimateRemoveLiquidityOneToken)]
    fn estimate_remove_liquidity_one_token(
        &self,
        shares: BigUint,
        token_out: TokenIdentifier,
    ) -> BigUint {
        let index_token_out = self.get_token_index(&token_out);

        let amount_out = self.do_remove_liquidity_one_token(shares, index_token_out, true);

        amount_out
    }

    #[view(getAnalyticsForLastEpochs)]
    fn get_analytics_for_last_epochs(
        &self,
        countback: u64,
    ) -> MultiValueEncoded<Self::Api, analytics::AnalyticsForEpoch<Self::Api>> {
        let tokens = (0..self.nb_tokens().get())
            .map(|i| self.tokens(i).get())
            .collect();

        let res = self.do_get_analytics_for_last_epochs(countback, tokens);

        res.into()
    }

    #[view(getStatus)]
    fn get_status(&self) -> PairStatus<Self::Api> {
        let prev_epoch = self.blockchain().get_block_epoch() - 1u64;

        let nb_tokens = self.nb_tokens().get();

        let tokens: ManagedVec<Self::Api, TokenIdentifier> = (0..nb_tokens)
            .into_iter()
            .map(|i| self.tokens(i).get())
            .collect();

        let reserves: ManagedVec<Self::Api, BigUint> = (0..nb_tokens)
            .into_iter()
            .map(|i| self.reserves(i).get())
            .collect();

        let opt_platform_fees_receiver = if self.platform_fees_receiver().is_empty() {
            Option::None
        } else {
            Option::Some(self.platform_fees_receiver().get())
        };

        let mut volume_prev_epoch = ManagedVec::<Self::Api, BigUint>::new();
        let mut fees_prev_epoch = ManagedVec::<Self::Api, BigUint>::new();
        for token in tokens.iter() {
            volume_prev_epoch.push(self.trading_volume(prev_epoch, &token).get());
            fees_prev_epoch.push(self.lp_fees(prev_epoch, &token).get());
        }

        let mut fees_last_7_epochs = ManagedVec::<Self::Api, BigUint>::new();
        for token in tokens.iter() {
            let mut sum_lp_fees = BigUint::zero();
            for i in 0u64..=6u64 {
                sum_lp_fees += self.lp_fees(prev_epoch - i, &token).get();
            }
            fees_last_7_epochs.push(sum_lp_fees);
        }

        let status = PairStatus {
            paused: self.is_paused().get(),
            amp_factor: self.amp_factor().get(),
            nb_tokens,
            tokens,
            reserves,
            lp_token_identifier: self.lp_token().get(),
            lp_token_supply: self.lp_token_supply().get(),
            owner: self.blockchain().get_owner_address(),
            swap_fee: self.swap_fee().get(),
            platform_fees_receiver: opt_platform_fees_receiver,
            volume_prev_epoch,
            fees_prev_epoch,
            fees_last_7_epochs,
        };

        status
    }

    //
    // Callbacks
    //

    #[callback]
    fn lp_token_issue_callback(
        &self,
        caller: &ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        let (token_id, returned_tokens) = self.call_value().egld_or_single_fungible_esdt();
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let esdt = token_id.unwrap_esdt();
                self.lp_token().set(&esdt);
                self.send().direct_esdt(caller, &esdt, 0, &returned_tokens);
            }
            ManagedAsyncCallResult::Err(_) => {
                if token_id.is_egld() && returned_tokens > 0u64 {
                    self.send().direct_egld(caller, &returned_tokens);
                }
            }
        }
    }

    //
    // Functions
    //

    fn get_token_index(&self, token: &TokenIdentifier) -> usize {
        let mut found = false;
        let mut index_ = 0usize;

        for i in 0..self.nb_tokens().get() {
            if &self.tokens(i).get() == token {
                index_ = i;
                found = true;
                break;
            }
        }
        require!(found, "Invalid token");

        index_
    }

    //
    // Storage
    //

    #[storage_mapper("tokens")]
    fn tokens(&self, i: usize) -> SingleValueMapper<TokenIdentifier>;
}
