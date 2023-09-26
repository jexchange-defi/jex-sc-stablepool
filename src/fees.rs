multiversx_sc::imports!();

const FEE_DENOMINATOR: u64 = 1_000000u64;

#[multiversx_sc::module]
pub trait FeesModule {
    fn calculate_liquidity_fee(&self, amount: &BigUint) -> BigUint {
        return amount * self.liquidity_fee().get() / FEE_DENOMINATOR;
    }

    fn calculate_swap_fee(&self, amount: &BigUint) -> BigUint {
        return amount * self.swap_fee().get() / FEE_DENOMINATOR;
    }

    fn update_swap_fee(&self, nb_tokens: usize, swap_fee: u32) {
        self.swap_fee().set(swap_fee);

        let liquidity_fee = (swap_fee * nb_tokens as u32) / (4u32 * (nb_tokens as u32 - 1));

        self.liquidity_fee().set(&liquidity_fee);
    }

    #[storage_mapper("liquidity_fee")]
    fn liquidity_fee(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("swap_fee")]
    fn swap_fee(&self) -> SingleValueMapper<u32>;
}
