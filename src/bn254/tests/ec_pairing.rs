pub mod test {
    use std::fs::File;
    use std::io::Read;
    use std::sync::Arc;

    use boojum::cs::traits::cs::ConstraintSystem;
    use boojum::gadgets::boolean::Boolean;
    use boojum::pairing::bn256::G2Affine;
    use lazy_static::lazy_static;
    use serde::{Deserialize, Serialize};

    use boojum::field::goldilocks::GoldilocksField;
    use boojum::gadgets::curves::bn256::ec_mul::{
        width_4_windowed_multiplication, ScalarDecomposition,
    };
    use boojum::gadgets::curves::bn256::{BN256BaseNNField, BN256SWProjectivePointTwisted, BN256ScalarNNField};
    use boojum::gadgets::curves::sw_projective::SWProjectivePoint;
    use boojum::gadgets::traits::witnessable::WitnessHookable;
    use boojum::pairing::ff::{Field, PrimeField};
    use boojum::pairing::{CurveAffine, CurveProjective};

    use crate::bn254::tests::json::{DECOMPOSITION_TEST_CASES, EC_MUL_TEST_CASES, G2_CURVE_TEST_CASES};
    use crate::bn254::tests::types::{
        bn254_base_field_params, bn254_scalar_field_params, create_test_cs,
    };
    use crate::bn254::{BN256Affine, BN256Fq, BN256Fr};

    type F = GoldilocksField;
    type P = GoldilocksField;

    fn assert_equal_g2_points<CS>(
        cs: &mut CS,
        point: &mut BN256SWProjectivePointTwisted<F>,
        expected: &mut BN256SWProjectivePointTwisted<F>,
    ) where 
    CS: ConstraintSystem<F>{
        // Converting to affine representation
        let default_point = G2Affine::one();
        let ((x1, y1), is_infty1) = point.convert_to_affine_or_default(cs, default_point);
        let ((x2, y2), is_infty2) = expected.convert_to_affine_or_default(cs, default_point);

        // Enforcing point not to be at infinity
        let boolean_false = Boolean::allocated_constant(cs, false);
        Boolean::enforce_equal(cs, &is_infty1, &boolean_false);
        Boolean::enforce_equal(cs, &is_infty2, &boolean_false);
        
        // Enforcing x coordinates to be equal
        let x1_c0 = x1.witness_hook(cs)().unwrap().0.get();
        let x1_c1 = x1.witness_hook(cs)().unwrap().1.get();
        let x2_c0 = x2.witness_hook(cs)().unwrap().0.get();
        let x2_c1 = x2.witness_hook(cs)().unwrap().1.get();    
        assert!(x1_c0 == x2_c0 && x1_c1 == x2_c1, "x coordinates are not equal");
        
        // Enforcing y coordinates to be equal
        let y1_c0 = y1.witness_hook(cs)().unwrap().0.get();
        let y1_c1 = y1.witness_hook(cs)().unwrap().1.get();
        let y2_c0 = y2.witness_hook(cs)().unwrap().0.get();
        let y2_c1 = y2.witness_hook(cs)().unwrap().1.get();
        assert!(y1_c0 == y2_c0 && y1_c1 == y2_c1, "y coordinates are not equal");
    }

    #[test]
    fn test_g2_curve() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;

        // Running tests from file
        for (i, test) in G2_CURVE_TEST_CASES.tests.iter().enumerate() {
            // Getting points
            let mut point_1 = test.point_1.to_projective_point(cs);
            let mut point_2 = test.point_2.to_projective_point(cs);

            let point_2_x = point_2.x.clone();
            let point_2_y = point_2.y.clone();
            
            // Getting real results
            let mut sum = point_1.add_mixed(cs, &mut (point_2_x, point_2_y));
            let mut point_1_double = point_1.double(cs);
            let mut point_2_double = point_2.double(cs);

            // Getting expected results
            let mut expected_sum = test.expected.sum.to_projective_point(cs);
            let mut expected_point_1_double = test.expected.point_1_double.to_projective_point(cs);
            let mut expected_point_2_double = test.expected.point_2_double.to_projective_point(cs);

            // Asserting points to be equal
            assert_equal_g2_points(cs, &mut sum, &mut expected_sum);
            assert_equal_g2_points(cs, &mut point_1_double, &mut expected_point_1_double);
            assert_equal_g2_points(cs, &mut point_2_double, &mut expected_point_2_double);

            // Print a message every 10 tests
            if i % 2 == 1 {
                println!("G2 tests {} and {} have passed", i, i + 1);
            }
        }
    }
}
