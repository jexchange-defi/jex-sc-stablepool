multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait AccessControlModule {
    //
    // Functions
    //

    fn do_allow_sc(&self, address: &ManagedAddress, allow: bool) {
        if allow {
            self.is_sc_allowed(address).set(true);
        } else {
            self.is_sc_allowed(address).clear();
        }
    }

    fn do_enable_access_control(&self) {
        self.is_ac_enabled().set(true);
    }

    fn do_disable_access_control(&self) {
        self.is_ac_enabled().clear();
    }

    fn require_caller_is_authorized(&self) {
        if self.is_ac_enabled().get() {
            let caller = self.blockchain().get_caller();

            if self.blockchain().is_smart_contract(&caller) {
                require!(self.is_sc_allowed(&caller).get(), "Unauthorized");
            }
        }
    }

    //
    // Storage
    //

    #[storage_mapper("is_ac_enabled")]
    fn is_ac_enabled(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("is_sc_allowed")]
    fn is_sc_allowed(&self, address: &ManagedAddress) -> SingleValueMapper<bool>;
}
