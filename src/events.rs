multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode)]
pub struct SwapEvent<M: ManagedTypeApi> {
    caller: ManagedAddress<M>,
    token_in: TokenIdentifier<M>,
    amount_in: BigUint<M>,
    token_out: TokenIdentifier<M>,
    net_amount_out: BigUint<M>,
    lp_fee_amount: BigUint<M>,
    platform_fee_amount: BigUint<M>,
    timestamp: u64,
}

#[multiversx_sc::module]
pub trait EventsModule {
    fn emit_swap_event(
        &self,
        token_in: TokenIdentifier,
        amount_in: BigUint,
        token_out: TokenIdentifier,
        net_amount_out: BigUint,
        lp_fee_amount: BigUint,
        platform_fee_amount: BigUint,
    ) {
        self.swap_event(
            &token_in,
            &token_out,
            &SwapEvent {
                caller: self.blockchain().get_caller(),
                token_in: token_in.clone(),
                amount_in,
                token_out: token_out.clone(),
                net_amount_out,
                lp_fee_amount,
                platform_fee_amount,
                timestamp: self.blockchain().get_block_timestamp(),
            },
        )
    }

    #[event("swap")]
    fn swap_event(
        &self,
        #[indexed] token_in: &TokenIdentifier,
        #[indexed] token_out: &TokenIdentifier,
        swap_event: &SwapEvent<Self::Api>,
    );
}
