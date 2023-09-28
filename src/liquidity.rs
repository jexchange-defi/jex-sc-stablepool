multiversx_sc::imports!();

const LP_TOKEN_DECIMALS: u32 = 18u32;

#[multiversx_sc::module]
pub trait LiquidityModule:
    super::amm::AmmModule + super::fees::FeesModule + super::maths::MathsModule
{
    // Estimate value of 1 share
    // How many tokens is one share worth?
    #[view(getVirtualPrice)]
    fn get_virtual_price(&self) -> BigUint {
        let xp = self.get_xp();
        let d = self.amm_get_d(&xp);
        let total_supply = self.lp_token_supply().get();

        if total_supply > 0 {
            (d * BigUint::from(10u32).pow(LP_TOKEN_DECIMALS)) / total_supply
        } else {
            BigUint::zero()
        }
    }

    /// Calculate amount of token i to receive for shares
    ///
    /// shares: Shares to burn
    /// i: Index of token to withdraw
    ///
    /// return (dy, fee)
    /// dy: Amount of token i to receive
    /// fee: Fee for withdraw. Fee already included in dy
    fn calculate_withdraw_one_token(&self, shares: &BigUint, i: usize) -> (BigUint, BigUint) {
        let total_supply = self.lp_token_supply().get();
        let n = self.nb_tokens().get();

        let xp = self.get_xp();

        // Calculate d0 and d1
        let d0 = self.amm_get_d(&xp);
        let d1 = &d0 - &((&d0 * shares) / &total_supply);

        // Calculate reduction in y if D = d1
        let y0 = self.amm_get_yd(i, &xp, &d1);
        // d1 <= d0 so y must be <= xp[i]
        let dy0 = (xp.get(i).clone_value() - &y0) / self.multipliers(i).get();

        // Calculate imbalance fee, update xp with fees
        let mut new_xs = ManagedVec::<Self::Api, BigUint>::new();
        for j in 0..n {
            let xpj = xp.get(j).clone_value();
            let dx = if j == i {
                (&xpj * &d1) / &d0 - &y0
            } else {
                // d1 / d0 <= 1
                &xpj - &((&xpj * &d1) / &d0)
            };

            let fee = self.calculate_liquidity_fee(&dx);
            new_xs.push(xpj - fee);
        }

        // Recalculate y with xp including imbalance fees
        let y1 = self.amm_get_yd(i, &new_xs, &d1);
        // - 1 to round down
        let dy = (new_xs.get(i).clone_value() - y1 - 1u32) / self.multipliers(i).get();
        let fee = dy0 - &dy;

        (dy, fee)
    }

    fn do_add_liquidity(&self, amounts: ManagedVec<Self::Api, BigUint>, readonly: bool) -> BigUint {
        // calculate current liquidity d0
        let total_supply = self.lp_token_supply().get();
        let mut d0 = BigUint::zero();
        let old_xs = self.get_xp();
        if total_supply > 0 {
            d0 = self.amm_get_d(&old_xs);
        }

        // Transfer tokens in
        let n = old_xs.len();
        let mut new_xs = ManagedVec::<Self::Api, BigUint>::new();

        for i in 0..n {
            let amount = amounts.get(i).clone_value();
            if &amount > &0 {
                new_xs.push(old_xs.get(i).clone_value() + amount * self.multipliers(i).get());
            } else {
                new_xs.push(old_xs.get(i).clone_value());
            }
        }

        // Calculate new liquidity d1
        let d1 = self.amm_get_d(&new_xs);
        require!(&d1 > &d0, "liquidity didn't increase");

        // Recalcuate D accounting for fee on imbalance
        let mut new_xs2 = ManagedVec::<Self::Api, BigUint>::new();
        let d2: BigUint;
        if total_supply > 0 {
            for i in 0..n {
                // TODO: why old_xs[i] * d1 / d0? why not d1 / N?
                let ideal_balance = (old_xs.get(i).clone_value() * &d1) / &d0;
                let diff = self.abs_diff(&new_xs.get(i).clone_value(), &ideal_balance);
                let fee = self.calculate_liquidity_fee(&diff);
                let new_xi = new_xs.get(i).clone_value() - fee;
                new_xs2.push(new_xi);
            }

            d2 = self.amm_get_d(&new_xs2);
        } else {
            d2 = d1;
        }

        // Shares to mint = (d2 - d0) / d0 * total supply
        // d1 >= d2 >= d0
        let shares = if total_supply > 0 {
            ((d2 - &d0) * total_supply) / &d0
        } else {
            d2
        };

        if !readonly {
            self.lp_mint(&shares);

            // Update reserves
            for i in 0..n {
                let new_balance = self.reserves(i).get() + amounts.get(i).clone_value();
                self.reserves(i).set(&new_balance);
            }
        }

        shares
    }

    fn do_remove_liquidity(
        &self,
        shares: &BigUint,
        readonly: bool,
    ) -> ManagedVec<Self::Api, BigUint> {
        let total_supply = self.lp_token_supply().get();
        let n = self.nb_tokens().get();
        let mut amounts_out = ManagedVec::<Self::Api, BigUint>::new();

        for i in 0..n {
            let balance = self.reserves(i).get();
            let amount_out = (&balance * shares) / &total_supply;

            if !readonly {
                self.reserves(i).set(&(&balance - &amount_out));
            }
            amounts_out.push(amount_out);
        }

        if !readonly {
            self.lp_burn(&shares);
        }

        amounts_out
    }

    /// Swap dx amount of token i for token j
    ///
    /// i: Index of token in
    /// j: Index of token out
    /// dx: Token in amount
    ///
    /// return (dy, fee)
    fn do_swap(&self, i: usize, j: usize, dx: BigUint, readonly: bool) -> (BigUint, BigUint) {
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
        dy -= &fee;

        if !readonly {
            self.reserves(i).update(|x| *x += &dx);
            self.reserves(j).update(|x| *x -= &dy);
        }

        (dy, fee)
    }

    /// Withdraw liquidity in token i
    ///
    /// shares: Shares to burn
    /// i: Token to withdraw
    fn do_remove_liquidity_one_token(&self, shares: BigUint, i: usize, readonly: bool) -> BigUint {
        let (amount_out, _) = self.calculate_withdraw_one_token(&shares, i);

        if !readonly {
            self.reserves(i).update(|x| *x -= &amount_out);
            self.lp_burn(&shares);
        }

        amount_out
    }

    // Return precision-adjusted reserves, adjusted to 18 decimals
    fn get_xp(&self) -> ManagedVec<Self::Api, BigUint> {
        let mut xp = ManagedVec::<Self::Api, BigUint>::new();

        for i in 0..self.nb_tokens().get() {
            xp.push(self.reserves(i).get() * self.multipliers(i).get());
        }

        xp
    }

    fn lp_burn(&self, amount: &BigUint) {
        self.lp_token_supply().update(|x| *x -= amount);

        let lp_token = self.lp_token().get();
        self.send().esdt_local_burn(&lp_token, 0, amount);
    }

    fn lp_mint(&self, amount: &BigUint) {
        self.lp_token_supply().update(|x| *x += amount);

        let lp_token = self.lp_token().get();
        self.send().esdt_local_mint(&lp_token, 0, amount);
    }

    #[storage_mapper("reserves")]
    fn reserves(&self, i: usize) -> SingleValueMapper<BigUint>;

    #[storage_mapper("lp_token_supply")]
    fn lp_token_supply(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("lp_token")]
    fn lp_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("multipliers")]
    fn multipliers(&self, i: usize) -> SingleValueMapper<BigUint>;

    #[storage_mapper("nb_tokens")]
    fn nb_tokens(&self) -> SingleValueMapper<usize>;
}
