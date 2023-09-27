multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait AmmModule: super::maths::MathsModule {
    /// Calculate D, sum of reserves in a perfectly balanced pool
    /// If reserves of x_0, x_1, ... x_(n-1) then sum(x_i) = D
    ///
    /// xp: precision-adjusted reserves
    ///
    /// return D
    fn amm_get_d(&self, xp: &ManagedVec<Self::Api, BigUint>) -> BigUint {
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
            d = ((s.clone() * &a + &n_big * &p) * &d) / (&d * &(&a - 1u32) + p * (&n_big + 1u32));

            if self.abs_diff(&d, &d_prev) <= 1 {
                return d;
            }
        }

        sc_panic!("D didn't converge");
    }

    /// Calculate the new balance of token j given the new balance of token i
    /// i: Index of token in
    /// j: Index of token out
    /// x: New balance of token i
    /// amp_factor: A
    /// xp Current precision-adjusted reserves
    fn amm_get_y(
        &self,
        i: usize,
        j: usize,
        x: BigUint,
        xp: ManagedVec<Self::Api, BigUint>,
    ) -> BigUint {
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

        let d = self.amm_get_d(&xp);
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
    /// reserves xp and liquidity d
    ///
    /// i: Index of token to calculate the new balance
    /// xp: Precision-adjusted reserves
    /// d: Liquidity d
    ///
    /// return New balance of token i
    fn amm_get_yd(&self, i: usize, xp: &ManagedVec<Self::Api, BigUint>, d: &BigUint) -> BigUint {
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

    #[storage_mapper("amp_factor")]
    fn amp_factor(&self) -> SingleValueMapper<u32>;
}
