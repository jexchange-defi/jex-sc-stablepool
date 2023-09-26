multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait PausableModule {
    fn do_pause(&self) {
        self.is_paused().set(true);
    }

    fn do_unpause(&self) {
        self.is_paused().set(false);
    }

    fn require_not_paused(&self) {
        require!(!self.is_paused().get(), "Contract is paused");
    }

    #[view(isPaused)]
    #[storage_mapper("paused")]
    fn is_paused(&self) -> SingleValueMapper<bool>;
}
