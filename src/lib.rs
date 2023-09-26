#![no_std]

multiversx_sc::imports!();

mod pausable;

const LP_TOKEN_DECIMALS: u32 = 18u32;
const FEE_DENOMINATOR: u64 = 1_000000u64;
const SWAP_FEE: u64 = 300u64;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait JexScStablepoolContract: pausable::PausableModule {
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

    #[inline]
    fn abs_diff(&self, x: &BigUint, y: &BigUint) -> BigUint {
        if x >= y {
            x - y
        } else {
            y - x
        }
    }

    /// Calculate D, sum of balances in a perfectly balanced pool
    /// If balances of x_0, x_1, ... x_(n-1) then sum(x_i) = D
    /// xp: precision-adjusted balances
    /// return D
    fn get_d(&self, xp: &ManagedVec<Self::Api, BigUint>) -> BigUint {
        /*
        Newton's method to compute D
        -----------------------------
        f(D) = ADn^n + D^(n + 1) / (n^n prod(x_i)) - An^n sum(x_i) - D
        f'(D) = An^n + (n + 1) D^n / (n^n prod(x_i)) - 1

                     (as + np)D_n
        D_(n+1) = -----------------------
                  (a - 1)D_n + (n + 1)p

        a = An^n
        s = sum(x_i)
        p = (D_n)^(n + 1) / (n^n prod(x_i))
        */
        let n = xp.len();
        let n_big = BigUint::from(n);
        let a = &n_big * self.amp_factor().get();

        let mut s = BigUint::zero(); // x_0 + x_1 + ... + x_(n-1)
        for xp_i in xp.iter() {
            s += xp_i.clone_value();
        }

        // Newton's method
        // Initial guess, d <= s
        let mut d = s.clone();
        let mut d_prev: BigUint;
        for _ in 0..255 {
            // p = D^(n + 1) / (n^n * x_0 * ... * x_(n-1))
            let mut p = d.clone();

            for j in 0..n {
                p = (p * &d) / (&n_big * &xp.get(j).clone_value());
            }

            d_prev = d.clone();
            d = ((s.clone() * &a + &p * &n_big) * &d) / (&d * &(&a - 1u32) + p * (&n_big + 1u32));

            if self.abs_diff(&d, &d_prev) <= 1 {
                return d;
            }
        }

        sc_panic!("D didn't converge");
    }

    // Return precision-adjusted balances, adjusted to 18 decimals
    fn get_xp(&self) -> ManagedVec<Self::Api, BigUint> {
        let mut xp = ManagedVec::<Self::Api, BigUint>::new();

        for i in 0..self.nb_tokens().get() {
            xp.push(self.balances(i).get() * self.multipliers(i).get());
        }

        xp
    }

    /// Calculate the new balance of token j given the new balance of token i
    /// i Index of token in
    /// j Index of token out
    /// x New balance of token i
    /// xp Current precision-adjusted balances
    fn get_y(&self, i: usize, j: usize, x: BigUint, xp: ManagedVec<Self::Api, BigUint>) -> BigUint {
        /*
        Newton's method to compute y
        -----------------------------
        y = x_j

        f(y) = y^2 + y(b - D) - c

                    y_n^2 + c
        y_(n+1) = --------------
                   2y_n + b - D

        where
        s = sum(x_k), k != j
        p = prod(x_k), k != j
        b = s + D / (An^n)
        c = D^(n + 1) / (n^n * p * An^n)
        */
        let n = xp.len();
        let n_big = BigUint::from(n);
        let a = &n_big * self.amp_factor().get();

        let d = self.get_d(&xp);
        let mut s = BigUint::zero();
        let mut c = d.clone();

        let mut _x: BigUint;
        for k in 0..n {
            if k == i {
                _x = x.clone();
            } else if k == j {
                continue;
            } else {
                _x = xp.get(k).clone_value()
            };

            s += &_x;
            c = (c * &d) / (&n_big * &_x);
        }
        c = (c * &d) / (&n_big * &a);

        let b = s + &d / &a;

        // Newton's method
        let mut y_prev;

        // Initial guess, y <= d
        let mut y = d.clone();
        for _ in 0..255 {
            y_prev = y.clone();
            y = (&y * &y + &c) / (y * 2u32 + &b - &d);
            if self.abs_diff(&y, &y_prev) <= 1 {
                return y;
            }
        }

        sc_panic!("y didn't converge");
    }

    /// Calculate the new balance of token i given precision-adjusted
    /// balances xp and liquidity d
    ///
    /// i: Index of token to calculate the new balance
    /// xp: Precision-adjusted balances
    /// d: Liquidity d
    ///
    /// return New balance of token i
    fn get_yd(&self, i: usize, xp: &ManagedVec<Self::Api, BigUint>, d: &BigUint) -> BigUint {
        let n = xp.len();
        let n_big = BigUint::from(n);
        let a = &n_big * self.amp_factor().get();

        let mut s = BigUint::zero();
        let mut c = d.clone();

        let mut _x: BigUint;
        for k in 0..n {
            if k != i {
                _x = xp.get(k).clone_value();
            } else {
                continue;
            }

            s += &_x;
            c = (c * d) / (&n_big * &_x);
        }
        c = (c * d) / (&n_big * &a);
        let b = s + d / &a;

        // Newton's method
        let mut y_prev;
        // Initial guess, y <= d
        let mut y = d.clone();
        for _ in 0..255 {
            y_prev = y.clone();
            y = (&y * &y + &c) / (&y * 2u32 + &b - d);
            if self.abs_diff(&y, &y_prev) <= 1 {
                return y;
            }
        }

        sc_panic!("y didn't converge");
    }

    // Estimate value of 1 share
    // How many tokens is one share worth?
    #[view(getVirtualPrice)]
    fn get_virtual_price(&self) -> BigUint {
        let xp = self.get_xp();
        let d = self.get_d(&xp);
        let total_supply = self.lp_token_supply().get();

        if total_supply > 0 {
            (d * BigUint::from(10u32).pow(LP_TOKEN_DECIMALS)) / total_supply
        } else {
            BigUint::zero()
        }
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
        let y1 = self.get_y(i, j, x, xp);

        // y0 must be >= y1, since x has increased
        // -1 to round down
        let mut dy = (&y0 - &y1 - 1u32) / self.multipliers(j).get();

        // Subtract fee from dy
        let fee = (&dy * SWAP_FEE) / FEE_DENOMINATOR;
        dy -= fee;

        self.balances(i).update(|x| *x += &dx);
        self.balances(j).update(|x| *x -= &dy);

        dy
    }

    fn do_add_liquidity(&self, amounts: ManagedVec<Self::Api, BigUint>) -> BigUint {
        // calculate current liquidity d0
        let total_supply = self.lp_token_supply().get();
        let mut d0 = BigUint::zero();
        let old_xs = self.get_xp();
        if total_supply > 0 {
            d0 = self.get_d(&old_xs);
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
        let d1 = self.get_d(&new_xs);
        require!(&d1 > &d0, "liquidity didn't increase");

        // Recalcuate D accounting for fee on imbalance
        let liquidity_fee = self.liquidity_fee().get();
        let mut new_xs2 = ManagedVec::<Self::Api, BigUint>::new();
        let d2: BigUint;
        if total_supply > 0 {
            for i in 0..n {
                // TODO: why old_xs[i] * d1 / d0? why not d1 / N?
                let ideal_balance = (old_xs.get(i).clone_value() * &d1) / &d0;
                let diff = self.abs_diff(&new_xs.get(i).clone_value(), &ideal_balance);
                let fee = (diff * liquidity_fee) / FEE_DENOMINATOR;
                let new_xi = new_xs.get(i).clone_value() - fee;
                new_xs2.push(new_xi);
            }

            d2 = self.get_d(&new_xs2);
        } else {
            d2 = d1;
        }

        // Update balances
        for i in 0..n {
            let new_balance = self.balances(i).get() + amounts.get(i).clone_value();
            self.balances(i).set(&new_balance);
        }

        // Shares to mint = (d2 - d0) / d0 * total supply
        // d1 >= d2 >= d0
        let shares = if total_supply > 0 {
            ((d2 - &d0) * total_supply) / &d0
        } else {
            d2
        };

        self.lp_mint(&shares);

        shares
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
        let liquidity_fee = self.liquidity_fee().get();

        let xp = self.get_xp();

        // Calculate d0 and d1
        let d0 = self.get_d(&xp);
        let d1 = &d0 - &((&d0 * shares) / &total_supply);

        // Calculate reduction in y if D = d1
        let y0 = self.get_yd(i, &xp, &d1);
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

            new_xs.push(xpj - ((dx * liquidity_fee) / FEE_DENOMINATOR));
        }

        // Recalculate y with xp including imbalance fees
        let y1 = self.get_yd(i, &new_xs, &d1);
        // - 1 to round down
        let dy = (new_xs.get(i).clone_value() - y1 - 1u32) / self.multipliers(i).get();
        let fee = dy0 - &dy;

        (dy, fee)
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

    #[storage_mapper("amp_factor")]
    fn amp_factor(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("balances")]
    fn balances(&self, i: usize) -> SingleValueMapper<BigUint>;

    #[storage_mapper("liquidity_fee")]
    fn liquidity_fee(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("lp_token_supply")]
    fn lp_token_supply(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("lp_token")]
    fn lp_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("multipliers")]
    fn multipliers(&self, i: usize) -> SingleValueMapper<BigUint>;

    #[storage_mapper("nb_tokens")]
    fn nb_tokens(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("swap_fee")]
    fn swap_fee(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("tokens")]
    fn tokens(&self, i: usize) -> SingleValueMapper<TokenIdentifier>;
}
