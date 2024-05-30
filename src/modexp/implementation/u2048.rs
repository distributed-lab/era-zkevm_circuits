//! 256-byte implementation of the modular exponentiation algorithm.

use boojum::{
    crypto_bigint::U1024,
    cs::traits::cs::ConstraintSystem,
    field::SmallField,
    gadgets::{traits::selectable::Selectable, u2048::UInt2048},
};

const U2048_MAX_BITS: usize = 2048;
const U4096_MAX_BITS: usize = 4096;
const U2048_MAX_LIMBS: usize = 64;
const U4096_MAX_LIMBS: usize = 128;

/// Finds the result of exponentiating `base` to the power of `exponent` modulo `modulus`.
/// Input parameters format is done according to EIP-198:
/// https://eips.ethereum.org/EIPS/eip-198.
///
/// Implementation is based on _Algorithm 1_ from the paper
/// https://cse.buffalo.edu/srds2009/escs2009_submission_Gopal.pdf.
///
/// This implementation works with 256-byte `base`, `exponent`, and `modulus`.
pub fn modexp_256_bytes<F, CS>(
    cs: &mut CS,
    base: &UInt2048<F>,
    exponent: &UInt2048<F>,
    modulus: &UInt2048<F>,
) -> UInt2048<F>
where
    F: SmallField,
    CS: ConstraintSystem<F>,
{
    let mut a = UInt2048::allocated_constant(cs, (U1024::ONE, U1024::ZERO));
    let binary_expansion = exponent
        .to_le_bytes(cs)
        .into_iter()
        .map(|x| x.into_num().spread_into_bits::<CS, 8>(cs))
        .flatten()
        .collect::<Vec<_>>();

    for e in binary_expansion.into_iter().rev() {
        // a <- a^2 mod (modulus)
        let a_squared = a.modmul(cs, &a, modulus);

        // a <- a^2 * (base) mod (modulus)
        let a_base = a.modmul(cs, base, modulus);

        // If the i-th bit of the exponent is 1, then a <- a^2 * (base) mod (modulus)
        // Otherwise, we just set a <- a^2 mod (modulus)
        a = UInt2048::conditionally_select(cs, e, &a_base, &a_squared);
    }

    a
}
