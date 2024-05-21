use std::sync::Arc;
use arrayvec::ArrayVec;
use boojum::cs::traits::cs::ConstraintSystem;
use boojum::field::SmallField;
use boojum::gadgets::boolean::Boolean;
use boojum::gadgets::u256::UInt256;
use crate::bn254::BN256BaseNNFieldParams;
use crate::ethereum_types::U256;

/// Checks that each passed value is in BN256 base primary field.
/// Masks value if not.
pub(crate) fn validate_in_field<F: SmallField, CS: ConstraintSystem<F>, const N: usize>(
    cs: &mut CS,
    values: &mut [&mut UInt256<F>; N], // Changed to mutable references
    params: &Arc<BN256BaseNNFieldParams>,
) -> ArrayVec<Boolean<F>, N>
{
    let p_u256 = U256([
        params.modulus_u1024.as_ref().as_words()[0],
        params.modulus_u1024.as_ref().as_words()[1],
        params.modulus_u1024.as_ref().as_words()[2],
        params.modulus_u1024.as_ref().as_words()[3],
    ]);
    let p_u256 = UInt256::allocated_constant(cs, p_u256);

    let mut exception_flags = ArrayVec::<_, N>::new();

    let mut temp_values = vec![];

    for value in values.iter_mut() {
        let (_res, is_in_range) = value.overflowing_sub(cs, &p_u256);
        let masked_value = value.mask(cs, is_in_range);
        temp_values.push(masked_value); // Store new values temporarily
        let value_is_not_in_range = is_in_range.negated(cs);
        exception_flags.push(value_is_not_in_range);
    }

    // Now assign the new values back to the original references
    for (value, new_value) in values.iter_mut().zip(temp_values.into_iter()) {
        **value = new_value;
    }

    exception_flags
}

