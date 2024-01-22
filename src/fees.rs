multiversx_sc::imports!();

const FEE_DENOMINATOR: u64 = 1_000000u64;

#[multiversx_sc::module]
pub trait FeesModule {
    fn calculate_liquidity_fee(&self, amount: &BigUint) -> BigUint {
        return amount * self.liquidity_fee().get() / FEE_DENOMINATOR;
    }

    // return (lp fee, platform fee)
    fn calculate_swap_fee(&self, amount: &BigUint) -> (BigUint, BigUint) {
        let total_fee = amount * self.swap_fee().get() / FEE_DENOMINATOR;

        let platform_fee = &total_fee * 33u32 / 100u32;

        (&total_fee - &platform_fee, platform_fee)
    }

    fn do_configure_platform_fees_receiver(&self, receiver: &ManagedAddress) {
        self.platform_fees_receiver().set(receiver);
    }

    fn update_swap_fee(&self, nb_tokens: usize, swap_fee: u32) {
        self.swap_fee().set(swap_fee);

        let liquidity_fee = (swap_fee * nb_tokens as u32) / (4u32 * (nb_tokens as u32 - 1));

        self.liquidity_fee().set(&liquidity_fee);
    }

    #[view(getLiquidityFee)]
    #[storage_mapper("liquidity_fee")]
    fn liquidity_fee(&self) -> SingleValueMapper<u32>;

    #[view(getPlatformFeesReceiver)]
    #[storage_mapper("platform_fees_receiver")]
    fn platform_fees_receiver(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getSwapFee)]
    #[storage_mapper("swap_fee")]
    fn swap_fee(&self) -> SingleValueMapper<u32>;
}
