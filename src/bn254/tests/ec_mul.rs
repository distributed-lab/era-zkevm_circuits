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
    use crate::bn254::tests::types::{
        bn254_base_field_params, bn254_scalar_field_params, create_test_cs,
    };
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
        for (i, test) in DECOMPOSITION_TEST_CASES.tests.iter().enumerate() {
            let k = BN256Fr::from_str(&test.k).unwrap();
            let mut k = BN256ScalarNNField::allocate_checked(cs, k, &scalar_params);
            let decomposition = ScalarDecomposition::from(cs, &mut k, &scalar_params);

            let k1 = decomposition.k1.witness_hook(cs)().unwrap().get();
            let k1_was_negated = decomposition.k1_was_negated.witness_hook(cs)().unwrap();
            let k2 = decomposition.k2.witness_hook(cs)().unwrap().get();
            let k2_was_negated = decomposition.k2_was_negated.witness_hook(cs)().unwrap();

            let expected_k1 = BN256Fr::from_str(&test.k1).unwrap();
            let expected_k2 = BN256Fr::from_str(&test.k2).unwrap();

            assert_eq!(k1, expected_k1);
            assert_eq!(k1_was_negated, test.k1_negated);
            assert_eq!(k2, expected_k2);
            assert_eq!(k2_was_negated, test.k2_negated);

            // Print a message every 10 tests
            if i % 10 == 9 {
                println!("Decomposition tests {} to {} have passed", i - 8, i + 1);
            }
        }
    }

    fn assert_equal_g1_points<CS>(
        cs: &mut CS,
        point: &mut BN256SWProjectivePoint<F>,
        expected: &mut BN256SWProjectivePoint<F>,
    ) where
        CS: ConstraintSystem<F>,
    {
        // Converting to affine representation
        let default_point = BN256Affine::one();
        let ((x1, y1), is_infty1) = point.convert_to_affine_or_default(cs, default_point);
        let ((x2, y2), is_infty2) = expected.convert_to_affine_or_default(cs, default_point);

        // Enforcing point not to be at infinity
        let boolean_false = Boolean::allocated_constant(cs, false);
        Boolean::enforce_equal(cs, &is_infty1, &boolean_false);
        Boolean::enforce_equal(cs, &is_infty2, &boolean_false);

        // Enforcing x coordinates to be equal
        let x1 = x1.witness_hook(cs)().unwrap().get();
        let x2 = x2.witness_hook(cs)().unwrap().get();
        assert!(x1 == x2, "x coordinates are not equal");

        // Enforcing y coordinates to be equal
        let y1 = y1.witness_hook(cs)().unwrap().get();
        let y2 = y2.witness_hook(cs)().unwrap().get();
        assert!(y1 == y2, "y coordinates are not equal");
    }

    #[test]
    fn test_width_4_multiplication() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;
        let scalar_params = Arc::new(bn254_scalar_field_params());
        let base_params = Arc::new(bn254_base_field_params());

        for (i, test) in EC_MUL_TEST_CASES.tests.iter().enumerate() {
            // Define the base point
            let point_nn = test.point.to_projective_point(cs);

            // Define the scalar to multiply with.
            let scalar = BN256Fr::from_str(&test.scalar).unwrap();
            let scalar_nn = BN256ScalarNNField::allocate_checked(cs, scalar, &scalar_params);

            // Doing actual multiplication
            let mut actual = width_4_windowed_multiplication(
                cs,
                point_nn,
                scalar_nn,
                &base_params,
                &scalar_params,
            );
            let mut expected = test.expected.to_projective_point(cs);

            // Making assertion
            assert_equal_g1_points(cs, &mut actual, &mut expected);

            // Print a message every 3 tests
            if i % 2 == 1 {
                println!("EC multiplication tests {} to {} have passed", i, i + 1);
            }
        }
    }
}
