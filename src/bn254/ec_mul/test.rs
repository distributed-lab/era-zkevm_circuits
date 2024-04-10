pub mod test {
    use std::sync::Arc;

    use boojum::field::goldilocks::GoldilocksField;
    use boojum::gadgets::curves::bn256::ec_mul::{
        width_4_windowed_multiplication, ScalarDecomposition,
    };
    use boojum::gadgets::curves::bn256::{BN256BaseNNField, BN256ScalarNNField};
    use boojum::gadgets::curves::sw_projective::SWProjectivePoint;
    use boojum::gadgets::traits::witnessable::WitnessHookable;
    use boojum::pairing::ff::{Field, PrimeField};
    use boojum::pairing::{CurveAffine, CurveProjective};
    
    use crate::bn254::ec_mul::bn254_base_field_params;
    use crate::bn254::{ec_mul::bn254_scalar_field_params, test_utils::create_test_cs};
    use crate::bn254::{BN256Affine, BN256Fq, BN256Fr};
    
    type F = GoldilocksField;
    type P = GoldilocksField;

    #[test]
    fn test_scalar_decomposition() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;
        let scalar_params = Arc::new(bn254_scalar_field_params());

        // Testing the scalar decomposition on the scalar
        // k = 0x161a87df4ee5620c75acf8cf7b2f1547183bf7368e2956fcc42ae0e439200c20
        // Expect to get:
        // k1 = 56507221619152889206123336271969597712
        // k2 = -111366987256442598357055499258064695755

        let test_scalar = BN256Fr::from_str(
            "9997758448649743481679332046642653083029331058711609633943349318238462807072",
        )
        .unwrap();
        let mut test_scalar = BN256ScalarNNField::allocate_checked(cs, test_scalar, &scalar_params);
        let decomposition = ScalarDecomposition::from(cs, &mut test_scalar, &scalar_params);

        let k1 = decomposition.k1.witness_hook(cs)().unwrap().get();
        let k1_was_negated = decomposition.k1_was_negated.witness_hook(cs)().unwrap();
        let k2 = decomposition.k2.witness_hook(cs)().unwrap().get();
        let k2_was_negated = decomposition.k2_was_negated.witness_hook(cs)().unwrap();

        let expected_k1 = BN256Fr::from_str("56507221619152889206123336271969597712").unwrap();
        let expected_k2 = BN256Fr::from_str("111366987256442598357055499258064695755").unwrap();

        assert_eq!(k1, expected_k1);
        assert_eq!(k1_was_negated, false);
        assert_eq!(k2, expected_k2);
        assert_eq!(k2_was_negated, true);
    }

    #[test]
    fn test_width_4_multiplication() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;
        let scalar_params = Arc::new(bn254_scalar_field_params());
        let base_params = Arc::new(bn254_base_field_params());

        // Setting two seeds
        let mut seed_scalar = BN256Fr::multiplicative_generator().pow([1111]);
        let mut seed_base = BN256Fr::multiplicative_generator().pow([2222]);

        const TESTS_NUMBER: u8 = 16;
        for _ in 0..TESTS_NUMBER {
            // Define the base point
            let point_raw = BN256Affine::one().mul(seed_base).into_affine();
            let (x, y) = point_raw.into_xy_unchecked();

            // Converting to the non-native field
            let x_nn = BN256BaseNNField::allocate_checked(cs, x, &base_params);
            let y_nn = BN256BaseNNField::allocate_checked(cs, y, &base_params);
            let point_nn = SWProjectivePoint::from_xy_unchecked(cs, x_nn, y_nn);

            // Define the scalar to multiply with.
            let scalar_nn = BN256ScalarNNField::allocate_checked(cs, seed_scalar, &scalar_params);

            let mut actual = width_4_windowed_multiplication(
                cs,
                point_nn,
                scalar_nn,
                &base_params,
                &scalar_params,
            );
            let ((actual_x, actual_y), _) =
                actual.convert_to_affine_or_default(cs, BN256Affine::one());
            let actual_x = actual_x.witness_hook(cs)().unwrap().get();
            let actual_y = actual_y.witness_hook(cs)().unwrap().get();

            // Actual point is just a base multiplied by a scalar using G1Affine
            let expected = point_raw.mul(seed_scalar).into_affine();
            let (expected_x, expected_y) = expected.as_xy();
            assert_eq!(actual_x, *expected_x);
            assert_eq!(actual_y, *expected_y);

            // Updating seeds to continue testing
            seed_scalar.square();
            seed_base.square();
        }
    }
}
