multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(ManagedVecItem, TopEncode, TopDecode, TypeAbi)]
pub struct AnalyticsForEpoch<M: ManagedTypeApi> {
    epoch: u64,
    volume: ManagedVec<M, BigUint<M>>,
    lp_fees: ManagedVec<M, BigUint<M>>,
}

#[multiversx_sc::module]
pub trait AnalyticsModule {
    fn analytics_add_lp_fees(&self, token: &TokenIdentifier, vol: &BigUint) {
        let epoch = self.blockchain().get_block_epoch();

        self.lp_fees(epoch, token).update(|x| *x += vol);
    }

    fn analytics_add_volume(&self, token: &TokenIdentifier, vol: &BigUint) {
        let epoch = self.blockchain().get_block_epoch();

        self.trading_volume(epoch, token).update(|x| *x += vol);
    }

    #[view(getFees)]
    #[storage_mapper("an_lp_fees")]
    fn lp_fees(&self, epoch: u64, token: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getTradingVolume)]
    #[storage_mapper("an_t_vol")]
    fn trading_volume(&self, epoch: u64, token: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    fn do_get_analytics_for_last_epochs(
        &self,
        countback: u64,
        tokens: ManagedVec<TokenIdentifier>,
    ) -> ManagedVec<Self::Api, AnalyticsForEpoch<Self::Api>> {
        let mut res = ManagedVec::<Self::Api, AnalyticsForEpoch<Self::Api>>::new();

        let current_epoch = self.blockchain().get_block_epoch();
        for epoch in (current_epoch - countback)..current_epoch {
            let volume = tokens
                .into_iter()
                .map(|token| self.trading_volume(epoch, &token).get())
                .collect();
            let lp_fees = tokens
                .into_iter()
                .map(|token| self.lp_fees(epoch, &token).get())
                .collect();
            let item = AnalyticsForEpoch::<Self::Api> {
                epoch,
                volume,
                lp_fees,
            };
            res.push(item);
        }

        res
    }
}
