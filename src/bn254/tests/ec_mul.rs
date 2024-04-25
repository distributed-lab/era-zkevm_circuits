pub mod test {
    use std::fs::File;
    use std::io::Read;
    use std::sync::Arc;

    use boojum::cs::traits::cs::ConstraintSystem;
    use boojum::gadgets::boolean::Boolean;
    use lazy_static::lazy_static;
    use serde::{Deserialize, Serialize};

    use boojum::field::goldilocks::GoldilocksField;
    use boojum::gadgets::curves::bn256::ec_mul::{
        width_4_windowed_multiplication, ScalarDecomposition,
    };
    use boojum::gadgets::curves::bn256::{
        BN256BaseNNField, BN256SWProjectivePoint, BN256ScalarNNField,
    };
    use boojum::gadgets::curves::sw_projective::SWProjectivePoint;
    use boojum::gadgets::traits::witnessable::WitnessHookable;
    use boojum::pairing::ff::{Field, PrimeField};
    use boojum::pairing::{CurveAffine, CurveProjective};

    use crate::bn254::tests::json::{DECOMPOSITION_TEST_CASES, EC_MUL_TEST_CASES};
    use crate::bn254::tests::utils::assert::assert_equal_g1_points;
    use crate::bn254::tests::utils::cs::{
        bn254_base_field_params, bn254_scalar_field_params, create_test_cs,
    };
    use crate::bn254::tests::utils::debug_success;
    use crate::bn254::{BN256Affine, BN256Fq, BN256Fr};

    type F = GoldilocksField;
    type P = GoldilocksField;

    #[test]
    fn test_scalar_decomposition() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;
        let scalar_params = Arc::new(bn254_scalar_field_params());

        // Running tests from file
        const DEBUG_FREQUENCY: usize = 10;
        for (i, test) in DECOMPOSITION_TEST_CASES.tests.iter().enumerate() {
            // Input:
            let k = BN256Fr::from_str(&test.k).unwrap();
            let mut k = BN256ScalarNNField::allocate_checked(cs, k, &scalar_params);

            // Expected:
            let expected_k1 = BN256Fr::from_str(&test.k1).unwrap();
            let expected_k2 = BN256Fr::from_str(&test.k2).unwrap();

            // Actual:
            let decomposition = ScalarDecomposition::from(cs, &mut k, &scalar_params);
            let k1 = decomposition.k1.witness_hook(cs)().unwrap().get();
            let k1_was_negated = decomposition.k1_was_negated.witness_hook(cs)().unwrap();
            let k2 = decomposition.k2.witness_hook(cs)().unwrap().get();
            let k2_was_negated = decomposition.k2_was_negated.witness_hook(cs)().unwrap();

            // Asserting:
            assert_eq!(k1, expected_k1);
            assert_eq!(k1_was_negated, test.k1_negated);
            assert_eq!(k2, expected_k2);
            assert_eq!(k2_was_negated, test.k2_negated);

            debug_success("decomposition", i, DEBUG_FREQUENCY);
        }
    }

    #[test]
    fn test_width_4_multiplication() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;
        let scalar_params = Arc::new(bn254_scalar_field_params());
        let base_params = Arc::new(bn254_base_field_params());

        // Running tests from file
        const DEBUG_FREQUENCY: usize = 2;
        for (i, test) in EC_MUL_TEST_CASES.tests.iter().enumerate() {
            // Input:
            let point_nn = test.point.to_projective_point(cs);
            let scalar = BN256Fr::from_str(&test.scalar).unwrap();
            let scalar_nn = BN256ScalarNNField::allocate_checked(cs, scalar, &scalar_params);

            // Expected:
            let mut expected = test.expected.to_projective_point(cs);

            // Actual:
            let mut actual = width_4_windowed_multiplication(
                cs,
                point_nn,
                scalar_nn,
                &base_params,
                &scalar_params,
            );

            // Making assertion and debug success if OK
            assert_equal_g1_points(cs, &mut actual, &mut expected);
            debug_success("ec_mul", i, DEBUG_FREQUENCY);
        }
    }
}
