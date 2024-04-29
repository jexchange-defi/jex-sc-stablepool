#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait UnderlyingPriceSourceMock {
    #[init]
    fn init(&self) {}

    #[view(getExchangeRate)]
    fn get_exchange_rate(&self) -> BigUint<Self::Api> {
        self.exchange_rate().get()
    }

    #[storage_mapper("exchange_rate")]
    fn exchange_rate(&self) -> SingleValueMapper<BigUint>;
}
