#![no_std]

multiversx_sc::imports!();

mod amm;
mod fees;
mod liquidity;
mod maths;
mod pausable;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait JexScStablepoolContract:
    amm::AmmModule
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
        tokens_and_multipliers: MultiValueEncoded<(TokenIdentifier, BigUint)>,
    ) {
        self.amp_factor().set_if_empty(amp_factor);

        if self.nb_tokens().is_empty() {
            self.nb_tokens().set(tokens_and_multipliers.len());

            let mut i = 0usize;
            for (token, multiplier) in tokens_and_multipliers {
                self.tokens(i).set(&token);
                self.multipliers(i).set(&multiplier);
                i += 1;
            }
        }
    }

    //
    // Functions
    //

    /// Swap dx amount of token i for token j
    ///
    /// i: Index of token in
    /// j: Index of token out
    /// dx: Token in amount
    /// return dy
    fn do_swap(&self, i: usize, j: usize, dx: BigUint) -> BigUint {
        require!(i != j, "i = j");

        // Calculate dy
        let xp = self.get_xp();
        let x = xp.get(i).clone_value() + &dx * &self.multipliers(i).get();

        let y0 = xp.get(j).clone_value();
        let y1 = self.amm_get_y(i, j, x, xp);

        // y0 must be >= y1, since x has increased
        // -1 to round down
        let mut dy = (&y0 - &y1 - 1u32) / self.multipliers(j).get();

        // Subtract fee from dy
        let fee = self.calculate_swap_fee(&dy);
        dy -= fee;

        self.balances(i).update(|x| *x += &dx);
        self.balances(j).update(|x| *x -= &dy);

        dy
    }

    fn do_remove_liquidity(&self, shares: BigUint) -> ManagedVec<Self::Api, BigUint> {
        let total_supply = self.lp_token_supply().get();
        let n = self.nb_tokens().get();
        let mut amounts_out = ManagedVec::<Self::Api, BigUint>::new();

        for i in 0..n {
            let balance = self.balances(i).get();
            let amount_out = (&balance * &shares) / &total_supply;

            self.balances(i).set(&(&balance - &amount_out));
            amounts_out.push(amount_out);
        }

        self.lp_burn(&shares);

        amounts_out
    }

    /// Withdraw liquidity in token i
    ///
    /// shares: Shares to burn
    /// i: Token to withdraw
    fn remove_liquidity_one_token(&self, shares: BigUint, i: usize) -> BigUint {
        let (amount_out, _) = self.calculate_withdraw_one_token(&shares, i);

        self.balances(i).update(|x| *x -= &amount_out);
        self.lp_burn(&shares);

        amount_out
    }

    //
    // Storage
    //

    #[storage_mapper("tokens")]
    fn tokens(&self, i: usize) -> SingleValueMapper<TokenIdentifier>;
}
