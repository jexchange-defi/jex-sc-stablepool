multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait MathsModule {
    #[inline]
    fn abs_diff(&self, x: &BigUint, y: &BigUint) -> BigUint {
        if x >= y {
            x - y
        } else {
            y - x
        }
    }
}
