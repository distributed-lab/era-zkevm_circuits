pub mod test {
    use std::fs::File;
    use std::io::Read;
    use std::sync::Arc;

    use boojum::cs::traits::cs::ConstraintSystem;
    use boojum::gadgets::boolean::Boolean;
    use boojum::gadgets::curves::bn256::ec_pairing::{FinalExpEvaluation, LineFunctionEvaluation};
    use boojum::pairing::bn256::{Fq12, G2Affine};
    use lazy_static::lazy_static;
    use serde::{Deserialize, Serialize};

    use boojum::field::goldilocks::GoldilocksField;
    use boojum::gadgets::curves::bn256::ec_mul::{
        width_4_windowed_multiplication, ScalarDecomposition,
    };
    use boojum::gadgets::curves::bn256::{
        BN256BaseNNField, BN256Fq12NNField, BN256Fq2NNField, BN256Fq6NNField,
        BN256SWProjectivePointTwisted, BN256ScalarNNField,
    };
    use boojum::gadgets::curves::sw_projective::SWProjectivePoint;
    use boojum::gadgets::traits::witnessable::WitnessHookable;
    use boojum::pairing::ff::{Field, PrimeField};
    use boojum::pairing::{CurveAffine, CurveProjective};

    use crate::bn254::tests::json::{
        DECOMPOSITION_TEST_CASES, EC_MUL_TEST_CASES, FINAL_EXP_TEST_CASES, G2_CURVE_TEST_CASES,
        LINE_FUNCTION_TEST_CASES,
    };
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
        CS: ConstraintSystem<F>,
    {
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
        assert!(
            x1_c0 == x2_c0 && x1_c1 == x2_c1,
            "x coordinates are not equal"
        );

        // Enforcing y coordinates to be equal
        let y1_c0 = y1.witness_hook(cs)().unwrap().0.get();
        let y1_c1 = y1.witness_hook(cs)().unwrap().1.get();
        let y2_c0 = y2.witness_hook(cs)().unwrap().0.get();
        let y2_c1 = y2.witness_hook(cs)().unwrap().1.get();
        assert!(
            y1_c0 == y2_c0 && y1_c1 == y2_c1,
            "y coordinates are not equal"
        );
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

    fn assert_equal_fq2<CS: ConstraintSystem<F>>(
        cs: &mut CS,
        a: &BN256Fq2NNField<F>,
        b: &BN256Fq2NNField<F>,
        msg: &str,
    ) {
        let a_c0 = a.c0.witness_hook(cs)().unwrap().get();
        let a_c1 = a.c1.witness_hook(cs)().unwrap().get();

        let b_c0 = b.c0.witness_hook(cs)().unwrap().get();
        let b_c1 = b.c1.witness_hook(cs)().unwrap().get();

        let re_msg = format!("{}: Real parts are not equal", msg);
        let im_msg = format!("{}: Imaginary parts are not equal", msg);

        assert_eq!(a_c0, b_c0, "{}", re_msg);
        assert_eq!(a_c1, b_c1, "{}", im_msg);
    }

    fn assert_equal_fq6<CS: ConstraintSystem<F>>(
        cs: &mut CS,
        a: &BN256Fq6NNField<F>,
        b: &BN256Fq6NNField<F>,
        msg: &str,
    ) {
        assert_equal_fq2(cs, &a.c0, &b.c0, msg);
        assert_equal_fq2(cs, &a.c1, &b.c1, msg);
        assert_equal_fq2(cs, &a.c2, &b.c2, msg);
    }

    fn assert_equal_fq12<CS: ConstraintSystem<F>>(
        cs: &mut CS,
        a: &BN256Fq12NNField<F>,
        b: &BN256Fq12NNField<F>,
        msg: &str,
    ) {
        assert_equal_fq6(cs, &a.c0, &b.c0, msg);
        assert_equal_fq6(cs, &a.c1, &b.c1, msg);
    }

    #[test]
    fn test_line_functions() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;
        let base_field_params = Arc::new(bn254_base_field_params());

        // Running tests from file
        for (i, test) in LINE_FUNCTION_TEST_CASES.tests.iter().enumerate() {
            // Getting points
            let mut g2_point_1 = test.g2_point_1.to_projective_point(cs);
            let mut g2_point_2 = test.g2_point_2.to_projective_point(cs);
            let mut g1_point = test.g1_point.to_projective_point(cs);

            // Getting real results
            let mut line_add = LineFunctionEvaluation::zero(cs, &base_field_params);
            let line_add = line_add.at_line(cs, &mut g2_point_1, &mut g2_point_2, &mut g1_point);
            let (c0, c3, c4) = line_add.as_tuple();
            let line_add = BN256Fq12NNField::from_c0c3c4(cs, c0, c3, c4);

            let mut line_tangent_1 = LineFunctionEvaluation::zero(cs, &base_field_params);
            let line_tangent_1 = line_tangent_1.at_tangent(cs, &mut g2_point_1, &mut g1_point);
            let (c0, c3, c4) = line_tangent_1.as_tuple();
            let line_tangent_1 = BN256Fq12NNField::from_c0c3c4(cs, c0, c3, c4);

            let mut line_tangent_2 = LineFunctionEvaluation::zero(cs, &base_field_params);
            let line_tangent_2 = line_tangent_2.at_tangent(cs, &mut g2_point_2, &mut g1_point);
            let (c0, c3, c4) = line_tangent_2.as_tuple();
            let line_tangent_2 = BN256Fq12NNField::from_c0c3c4(cs, c0, c3, c4);

            // Asserting
            let expected_line_add = test.expected.line_add.to_fq12(cs);
            let expected_line_tangent_1 = test.expected.line_tangent_1.to_fq12(cs);
            let expected_line_tangent_2 = test.expected.line_tangent_2.to_fq12(cs);

            assert_equal_fq12(
                cs,
                &line_add,
                &expected_line_add,
                "Line functions are wrong",
            );
            assert_equal_fq12(
                cs,
                &line_tangent_1,
                &expected_line_tangent_1,
                "Line functions are wrong",
            );
            assert_equal_fq12(
                cs,
                &line_tangent_2,
                &expected_line_tangent_2,
                "Line functions are wrong",
            );

            // Print a message every 3 tests
            if i % 3 == 2 {
                println!("Line evaluation tests {} to {} have passed", i - 1, i + 1);
            }
        }
    }

    #[test]
    fn test_final_exponentiation() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;

        // Running tests from file
        for (i, test) in FINAL_EXP_TEST_CASES.tests.iter().enumerate() {
            // Expected:
            let expected_f_final = test.expected.to_fq12(cs);

            // Actual:
            let mut f = test.scalar.to_fq12(cs);
            let f_final = FinalExpEvaluation::evaluate(cs, &mut f);
            let f_final = f_final.get();

            assert_equal_fq12(
                cs,
                &f_final,
                &expected_f_final,
                "Final exponentiation is wrong",
            );

            // Print a message every 3 tests
            if i % 2 == 1 {
                println!("Final exponentiation tests {} to {} have passed", i, i + 1);
            }
        }
    }
}
