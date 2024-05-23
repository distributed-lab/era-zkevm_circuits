use std::sync::Arc;
use arrayvec::ArrayVec;
use boojum::cs::traits::cs::ConstraintSystem;
use boojum::field::SmallField;
use boojum::gadgets::boolean::Boolean;
use boojum::gadgets::non_native_field::implementations::NonNativeFieldOverU16Params;
use boojum::gadgets::u256::UInt256;

use boojum::pairing::ff::PrimeField;
use crate::bn254::{BN256BaseNNField, BN256BaseNNFieldParams, BN256Fq, BN256Fq2NNField};
use crate::ethereum_types::U256;

// B parameter for BN256 curve equation.
const B: &str = "3";

/// Checks that each passed value is in `BN256` primary field:
/// base or scalar depending on params.
/// Masks value in-place otherwise.
pub(crate) fn validate_in_field<F: SmallField, T: PrimeField, CS: ConstraintSystem<F>, const N: usize>(
    cs: &mut CS,
    values: &mut [&mut UInt256<F>; N],
    params: &Arc<NonNativeFieldOverU16Params<T, 17>>,
) -> ArrayVec<Boolean<F>, N>
{
    let p_u256 = U256([
        params.modulus_u1024.as_ref().as_words()[0],
        params.modulus_u1024.as_ref().as_words()[1],
        params.modulus_u1024.as_ref().as_words()[2],
        params.modulus_u1024.as_ref().as_words()[3],
    ]);
    let p_u256 = UInt256::allocated_constant(cs, p_u256);

    let mut exceptions = ArrayVec::<_, N>::new();

    for value in values.iter_mut() {
        let (_, is_in_range) = value.overflowing_sub(cs, &p_u256);
        **value = value.mask(cs, is_in_range);
        let is_not_in_range = is_in_range.negated(cs);
        exceptions.push(is_not_in_range);
    }

    exceptions
}

/// Checks that the passed point is on `BN256` curve.
/// The `Infinity` point is not counted as on curve.
// The Short Weierstrass equation of the curve is y^2 = x^3 + 3.
pub(crate) fn is_on_curve<F: SmallField, CS: ConstraintSystem<F>>(
    cs: &mut CS,
    point: (&BN256BaseNNField<F>, &BN256BaseNNField<F>),
    params: &Arc<BN256BaseNNFieldParams>,
) -> Boolean<F>{
    let (x, y) = point;

    let mut x = x.clone();
    let mut y = y.clone();

    let three = BN256Fq::from_str(B).unwrap();
    let mut three = BN256BaseNNField::allocated_constant(cs, three, params);

    let mut x_squared = x.square(cs);
    let mut x_cubed = x_squared.mul(cs, &mut x);

    let mut x_cubed_plus_three = x_cubed.add(cs, &mut three);
    let mut y_squared = y.double(cs);

    BN256BaseNNField::equals(cs, &mut y_squared, &mut x_cubed_plus_three)
}

/// Check whether passed point is classified as `Infinity`.
/// See https://eips.ethereum.org/EIPS/eip-196 for further details.
// We use `UInt256` instead of `BN256BaseNNField`
// because we need to be able to check the unmasked value.
pub(crate) fn is_affine_infinity<F: SmallField, CS: ConstraintSystem<F>>(
    cs: &mut CS,
    point: (&UInt256<F>, &UInt256<F>),
) -> Boolean<F> {
    let (mut x, mut y) = point;
    let x_is_zero = x.is_zero(cs);
    let y_is_zero = y.is_zero(cs);

    Boolean::multi_or(cs, &[x_is_zero, y_is_zero])
}
