use super::*;
use crate::base_structures::log_query::*;
use crate::base_structures::memory_query::*;
use crate::base_structures::precompile_input_outputs::PrecompileFunctionOutputData;
use crate::demux_log_queue::StorageLogQueue;
use crate::ethereum_types::U256;
use crate::fsm_input_output::circuit_inputs::INPUT_OUTPUT_COMMITMENT_LENGTH;
use crate::fsm_input_output::*;
use arrayvec::ArrayVec;
use boojum::algebraic_props::round_function::AlgebraicRoundFunction;
use boojum::crypto_bigint::{Zero, U1024};
use boojum::cs::gates::ConstantAllocatableCS;
use boojum::cs::traits::cs::ConstraintSystem;
use boojum::field::SmallField;
use boojum::gadgets::boolean::Boolean;
use boojum::gadgets::curves::sw_projective::SWProjectivePoint;
use boojum::gadgets::keccak256::keccak256;
use boojum::gadgets::non_native_field::implementations::*;
use boojum::gadgets::num::Num;
use boojum::gadgets::queue::CircuitQueueWitness;
use boojum::gadgets::queue::QueueState;
use boojum::gadgets::traits::allocatable::{CSAllocatableExt, CSPlaceholder};
use boojum::gadgets::traits::round_function::CircuitRoundFunction;
use boojum::gadgets::traits::selectable::Selectable;
use boojum::gadgets::traits::witnessable::WitnessHookable;
use boojum::gadgets::u16::UInt16;
use boojum::gadgets::u160::UInt160;
use boojum::gadgets::u256::UInt256;
use boojum::gadgets::u32::UInt32;
use boojum::gadgets::u8::UInt8;
use cs_derive::*;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use zkevm_opcode_defs::system_params::PRECOMPILE_AUX_BYTE;

pub mod input;

pub const MEMORY_QUERIES_PER_CALL: usize = 4;

// fn ecmul_precompile_inner_routine<
//     F: SmallField,
//     CS: ConstraintSystem<F>,
//     const MESSAGE_HASH_CAN_BE_ZERO: bool,
// >(
//     cs: &mut CS,
//     recid: &UInt8<F>,
//     r: &UInt256<F>,
//     s: &UInt256<F>,
//     message_hash: &UInt256<F>,
//     valid_x_in_external_field: Secp256BaseNNField<F>,
//     valid_y_in_external_field: Secp256BaseNNField<F>,
//     valid_t_in_external_field: Secp256BaseNNField<F>,
//     base_field_params: &Arc<Secp256BaseNNFieldParams>,
//     scalar_field_params: &Arc<Secp256ScalarNNFieldParams>,
// ) -> (Boolean<F>, UInt256<F>) {
//     use boojum::pairing::ff::Field;
//     let curve_b = Secp256Affine::b_coeff();

//     let mut minus_one = Secp256Fq::one();
//     minus_one.negate();

//     let mut curve_b_nn =
//         Secp256BaseNNField::<F>::allocated_constant(cs, curve_b, &base_field_params);
//     let mut minus_one_nn =
//         Secp256BaseNNField::<F>::allocated_constant(cs, minus_one, &base_field_params);

//     let secp_n_u256 = U256([
//         scalar_field_params.modulus_u1024.as_ref().as_words()[0],
//         scalar_field_params.modulus_u1024.as_ref().as_words()[1],
//         scalar_field_params.modulus_u1024.as_ref().as_words()[2],
//         scalar_field_params.modulus_u1024.as_ref().as_words()[3],
//     ]);
//     let secp_n_u256 = UInt256::allocated_constant(cs, secp_n_u256);

//     let secp_p_u256 = U256([
//         base_field_params.modulus_u1024.as_ref().as_words()[0],
//         base_field_params.modulus_u1024.as_ref().as_words()[1],
//         base_field_params.modulus_u1024.as_ref().as_words()[2],
//         base_field_params.modulus_u1024.as_ref().as_words()[3],
//     ]);
//     let secp_p_u256 = UInt256::allocated_constant(cs, secp_p_u256);

//     let mut exception_flags = ArrayVec::<_, EXCEPTION_FLAGS_ARR_LEN>::new();

//     // recid = (x_overflow ? 2 : 0) | (secp256k1_fe_is_odd(&r.y) ? 1 : 0)
//     // The point X = (x, y) we are going to recover is not known at the start, but it is strongly related to r.
//     // This is because x = r + kn for some integer k, where x is an element of the field F_q . In other words, x < q.
//     // (here n is the order of group of points on elleptic curve)
//     // For secp256k1 curve values of q and n are relatively close, that is,
//     // the probability of a random element of Fq being greater than n is about 1/{2^128}.
//     // This in turn means that the overwhelming majority of r determine a unique x, however some of them determine
//     // two: x = r and x = r + n. If x_overflow flag is set than x = r + n

//     let [y_is_odd, x_overflow, ..] =
//         Num::<F>::from_variable(recid.get_variable()).spread_into_bits::<_, 8>(cs);

//     // check convention s < N/2
//     let s_upper_bound =
//         UInt256::allocated_constant(cs, U256::from_str_radix(HALF_SUBGROUP_SIZE, 16).unwrap());
//     let (_, uf) = s.overflowing_sub(cs, &s_upper_bound);
//     let s_too_large = uf.negated(cs);
//     exception_flags.push(s_too_large);

//     let (r_plus_n, of) = r.overflowing_add(cs, &secp_n_u256);
//     let mut x_as_u256 = UInt256::conditionally_select(cs, x_overflow, &r_plus_n, &r);
//     let error = Boolean::multi_and(cs, &[x_overflow, of]);
//     exception_flags.push(error);

//     // we handle x separately as it is the only element of base field of a curve (not a scalar field element!)
//     // check that x < q - order of base point on Secp256 curve
//     // if it is not actually the case - mask x to be zero
//     let (_res, is_in_range) = x_as_u256.overflowing_sub(cs, &secp_p_u256);
//     x_as_u256 = x_as_u256.mask(cs, is_in_range);
//     let x_is_not_in_range = is_in_range.negated(cs);
//     exception_flags.push(x_is_not_in_range);

//     let mut x_fe = convert_uint256_to_field_element(cs, &x_as_u256, &base_field_params);

//     let (mut r_fe, r_is_zero) =
//         convert_uint256_to_field_element_masked(cs, &r, &scalar_field_params);
//     exception_flags.push(r_is_zero);
//     let (mut s_fe, s_is_zero) =
//         convert_uint256_to_field_element_masked(cs, &s, &scalar_field_params);
//     exception_flags.push(s_is_zero);

//     let (mut message_hash_fe, message_hash_is_zero) = if MESSAGE_HASH_CAN_BE_ZERO {
//         (
//             convert_uint256_to_field_element(cs, &message_hash, scalar_field_params),
//             Boolean::allocated_constant(cs, false),
//         )
//     } else {
//         convert_uint256_to_field_element_masked(cs, &message_hash, scalar_field_params)
//     };
//     exception_flags.push(message_hash_is_zero);

//     // curve equation is y^2 = x^3 + b
//     // we compute t = r^3 + b and check if t is a quadratic residue or not.
//     // we do this by computing Legendre symbol (t, p) = t^[(p-1)/2] (mod p)
//     //           p = 2^256 - 2^32 - 2^9 - 2^8 - 2^7 - 2^6 - 2^4 - 1
//     // n = (p-1)/2 = 2^255 - 2^31 - 2^8 - 2^7 - 2^6 - 2^5 - 2^3 - 1
//     // we have to compute t^b = t^{2^255} / ( t^{2^31} * t^{2^8} * t^{2^7} * t^{2^6} * t^{2^5} * t^{2^3} * t)
//     // if t is not a quadratic residue we return error and replace x by another value that will make
//     // t = x^3 + b a quadratic residue

//     let mut t = x_fe.square(cs);
//     t = t.mul(cs, &mut x_fe);
//     t = t.add(cs, &mut curve_b_nn);

//     let t_is_zero = t.is_zero(cs);
//     exception_flags.push(t_is_zero);

//     // if t is zero then just mask
//     let t = Selectable::conditionally_select(cs, t_is_zero, &valid_t_in_external_field, &t);

//     // array of powers of t of the form t^{2^i} starting from i = 0 to 255
//     let mut t_powers = Vec::with_capacity(X_POWERS_ARR_LEN);
//     t_powers.push(t);

//     for _ in 1..X_POWERS_ARR_LEN {
//         let prev = t_powers.last_mut().unwrap();
//         let next = prev.square(cs);
//         t_powers.push(next);
//     }

//     let mut acc = t_powers[0].clone();
//     for idx in [3, 5, 6, 7, 8, 31].into_iter() {
//         let other = &mut t_powers[idx];
//         acc = acc.mul(cs, other);
//     }
//     let mut legendre_symbol = t_powers[255].div_unchecked(cs, &mut acc);

//     // we can also reuse the same values to compute square root in case of p = 3 mod 4
//     //           p = 2^256 - 2^32 - 2^9 - 2^8 - 2^7 - 2^6 - 2^4 - 1
//     // n = (p+1)/4 = 2^254 - 2^30 - 2^7 - 2^6 - 2^5 - 2^4 - 2^2

//     let mut acc_2 = t_powers[2].clone();
//     for idx in [4, 5, 6, 7, 30].into_iter() {
//         let other = &mut t_powers[idx];
//         acc_2 = acc_2.mul(cs, other);
//     }

//     let mut may_be_recovered_y = t_powers[254].div_unchecked(cs, &mut acc_2);
//     may_be_recovered_y.normalize(cs);
//     let may_be_recovered_y_negated = may_be_recovered_y.negated(cs);

//     if crate::config::CIRCUIT_VERSOBE {
//         dbg!(may_be_recovered_y.witness_hook(cs)());
//         dbg!(may_be_recovered_y_negated.witness_hook(cs)());
//     }

//     let [lowest_bit, ..] =
//         Num::<F>::from_variable(may_be_recovered_y.limbs[0]).spread_into_bits::<_, 16>(cs);

//     // if lowest bit != parity bit, then we need conditionally select
//     let should_swap = lowest_bit.xor(cs, y_is_odd);
//     let may_be_recovered_y = Selectable::conditionally_select(
//         cs,
//         should_swap,
//         &may_be_recovered_y_negated,
//         &may_be_recovered_y,
//     );

//     let t_is_nonresidue =
//         Secp256BaseNNField::<F>::equals(cs, &mut legendre_symbol, &mut minus_one_nn);
//     exception_flags.push(t_is_nonresidue);
//     // unfortunately, if t is found to be a quadratic nonresidue, we can't simply let x to be zero,
//     // because then t_new = 7 is again a quadratic nonresidue. So, in this case we let x to be 9, then
//     // t = 16 is a quadratic residue
//     let x =
//         Selectable::conditionally_select(cs, t_is_nonresidue, &valid_x_in_external_field, &x_fe);
//     let y = Selectable::conditionally_select(
//         cs,
//         t_is_nonresidue,
//         &valid_y_in_external_field,
//         &may_be_recovered_y,
//     );

//     // we recovered (x, y) using curve equation, so it's on curve (or was masked)
//     let mut r_fe_inversed = r_fe.inverse_unchecked(cs);
//     let mut s_by_r_inv = s_fe.mul(cs, &mut r_fe_inversed);
//     let mut message_hash_by_r_inv = message_hash_fe.mul(cs, &mut r_fe_inversed);

//     s_by_r_inv.normalize(cs);
//     let mut message_hash_by_r_inv_negated = message_hash_by_r_inv.negated(cs);
//     message_hash_by_r_inv_negated.normalize(cs);

//     // now we are going to compute the public key Q = (x, y) determined by the formula:
//     // Q = (s * X - hash * G) / r which is equivalent to r * Q = s * X - hash * G

//     if crate::config::CIRCUIT_VERSOBE {
//         dbg!(x.witness_hook(cs)());
//         dbg!(y.witness_hook(cs)());
//         dbg!(s_by_r_inv.witness_hook(cs)());
//         dbg!(message_hash_by_r_inv_negated.witness_hook(cs)());
//     }

//     let recovered_point =
//         SWProjectivePoint::<F, Secp256Affine, Secp256BaseNNField<F>>::from_xy_unchecked(cs, x, y);

//     // now we do multiplication
//     let mut s_times_x = width_4_windowed_multiplication(
//         cs,
//         recovered_point.clone(),
//         s_by_r_inv.clone(),
//         &base_field_params,
//         &scalar_field_params,
//     );

//     // let mut s_times_x = wnaf_scalar_mul(
//     //     cs,
//     //     recovered_point.clone(),
//     //     s_by_r_inv.clone(),
//     //     &base_field_params,
//     //     &scalar_field_params,
//     // );

//     let mut hash_times_g = fixed_base_mul(cs, message_hash_by_r_inv_negated, &base_field_params);
//     // let mut hash_times_g = fixed_base_mul(cs, message_hash_by_r_inv, &base_field_params);

//     let (mut q_acc, is_infinity) =
//         hash_times_g.convert_to_affine_or_default(cs, Secp256Affine::one());
//     let q_acc_added = s_times_x.add_mixed(cs, &mut q_acc);
//     let mut q_acc = Selectable::conditionally_select(cs, is_infinity, &s_times_x, &q_acc_added);

//     let ((q_x, q_y), is_infinity) = q_acc.convert_to_affine_or_default(cs, Secp256Affine::one());
//     exception_flags.push(is_infinity);
//     let any_exception = Boolean::multi_or(cs, &exception_flags[..]);

//     let zero_u8 = UInt8::zero(cs);

//     if crate::config::CIRCUIT_VERSOBE {
//         dbg!(q_x.witness_hook(cs)());
//         dbg!(q_y.witness_hook(cs)());
//     }

//     let mut bytes_to_hash = [zero_u8; 64];
//     let it = q_x.limbs[..16]
//         .iter()
//         .rev()
//         .chain(q_y.limbs[..16].iter().rev());

//     for (dst, src) in bytes_to_hash.array_chunks_mut::<2>().zip(it) {
//         let limb = unsafe { UInt16::from_variable_unchecked(*src) };
//         *dst = limb.to_be_bytes(cs);
//     }

//     let mut digest_bytes = keccak256(cs, &bytes_to_hash);
//     // digest is 32 bytes, but we need only 20 to recover address
//     digest_bytes[0..12].copy_from_slice(&[zero_u8; 12]); // empty out top bytes
//     digest_bytes.reverse();
//     let written_value_unmasked = UInt256::from_le_bytes(cs, digest_bytes);

//     let written_value = written_value_unmasked.mask_negated(cs, any_exception);
//     let all_ok = any_exception.negated(cs);

//     (all_ok, written_value)
// }

pub mod test {
    use boojum::{
        config::DevCSConfig,
        cs::{
            cs_builder::{new_builder, CsBuilder, CsBuilderImpl},
            cs_builder_reference::CsReferenceImplementationBuilder,
            gates::{
                BooleanConstraintGate, ConstantsAllocatorGate, DotProductGate,
                FmaGateInBaseFieldWithoutConstant, NopGate, ReductionGate, SelectionGate,
                U8x4FMAGate, UIntXAddGate, ZeroCheckGate,
            },
            implementations::reference_cs::CSReferenceImplementation,
            traits::{cs::ConstraintSystem, gate::GatePlacementStrategy},
            CSGeometry, GateConfigurationHolder, LookupParameters, StaticToolboxHolder,
        },
        field::{goldilocks::GoldilocksField, SmallField},
        gadgets::tables::{
            create_and8_table, create_byte_split_table, create_xor8_table, And8Table,
            ByteSplitTable, Xor8Table,
        },
    };

    use crate::ecrecover::secp256k1::fixed_base_mul_table::{
        create_fixed_base_mul_table, FixedBaseMulTable,
    };

    type F = GoldilocksField;
    type P = GoldilocksField;

    fn create_cs(
        max_trace_len: usize,
    ) -> CSReferenceImplementation<
        F,
        P,
        DevCSConfig,
        impl GateConfigurationHolder<F>,
        impl StaticToolboxHolder,
    > {
        let geometry = CSGeometry {
            num_columns_under_copy_permutation: 100,
            num_witness_columns: 0,
            num_constant_columns: 8,
            max_allowed_constraint_degree: 4,
        };
        let max_variables = 1 << 26;

        fn configure<
            F: SmallField,
            T: CsBuilderImpl<F, T>,
            GC: GateConfigurationHolder<F>,
            TB: StaticToolboxHolder,
        >(
            builder: CsBuilder<T, F, GC, TB>,
        ) -> CsBuilder<T, F, impl GateConfigurationHolder<F>, impl StaticToolboxHolder> {
            let builder = builder.allow_lookup(
                LookupParameters::UseSpecializedColumnsWithTableIdAsConstant {
                    width: 3,
                    num_repetitions: 8,
                    share_table_id: true,
                },
            );
            let builder = U8x4FMAGate::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = ConstantsAllocatorGate::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = FmaGateInBaseFieldWithoutConstant::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = ReductionGate::<F, 4>::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            // let owned_cs = ReductionGate::<F, 4>::configure_for_cs(owned_cs, GatePlacementStrategy::UseSpecializedColumns { num_repetitions: 8, share_constants: true });
            let builder = BooleanConstraintGate::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = UIntXAddGate::<32>::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = UIntXAddGate::<16>::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = UIntXAddGate::<8>::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = SelectionGate::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = ZeroCheckGate::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
                false,
            );
            let builder = DotProductGate::<4>::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            // let owned_cs = DotProductGate::<4>::configure_for_cs(owned_cs, GatePlacementStrategy::UseSpecializedColumns { num_repetitions: 1, share_constants: true });
            let builder = NopGate::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );

            builder
        }

        let builder_impl =
            CsReferenceImplementationBuilder::<F, P, DevCSConfig>::new(geometry, max_trace_len);
        let builder = new_builder::<_, F>(builder_impl);

        let builder = configure(builder);
        let mut owned_cs = builder.build(max_variables);

        // add tables
        let table = create_xor8_table();
        owned_cs.add_lookup_table::<Xor8Table, 3>(table);

        let table = create_and8_table();
        owned_cs.add_lookup_table::<And8Table, 3>(table);

        // let table = create_naf_abs_div2_table();
        // owned_cs.add_lookup_table::<NafAbsDiv2Table, 3>(table);

        // let table = create_wnaf_decomp_table();
        // owned_cs.add_lookup_table::<WnafDecompTable, 3>(table);

        seq_macro::seq!(C in 0..32 {
            let table = create_fixed_base_mul_table::<F, 0, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<0, C>, 3>(table);
            let table = create_fixed_base_mul_table::<F, 1, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<1, C>, 3>(table);
            let table = create_fixed_base_mul_table::<F, 2, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<2, C>, 3>(table);
            let table = create_fixed_base_mul_table::<F, 3, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<3, C>, 3>(table);
            let table = create_fixed_base_mul_table::<F, 4, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<4, C>, 3>(table);
            let table = create_fixed_base_mul_table::<F, 5, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<5, C>, 3>(table);
            let table = create_fixed_base_mul_table::<F, 6, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<6, C>, 3>(table);
            let table = create_fixed_base_mul_table::<F, 7, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<7, C>, 3>(table);
        });

        let table = create_byte_split_table::<F, 1>();
        owned_cs.add_lookup_table::<ByteSplitTable<1>, 3>(table);
        let table = create_byte_split_table::<F, 2>();
        owned_cs.add_lookup_table::<ByteSplitTable<2>, 3>(table);
        let table = create_byte_split_table::<F, 3>();
        owned_cs.add_lookup_table::<ByteSplitTable<3>, 3>(table);
        let table = create_byte_split_table::<F, 4>();
        owned_cs.add_lookup_table::<ByteSplitTable<4>, 3>(table);

        owned_cs
    }
}
