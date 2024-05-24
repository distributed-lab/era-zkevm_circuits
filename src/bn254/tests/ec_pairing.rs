pub mod test {

    use boojum::field::goldilocks::GoldilocksField;

    type F = GoldilocksField;
    type P = GoldilocksField;

    /// Tests whether G2 curve operations are correct. Namely, we verify:
    ///
    /// 1. The sum of two points.
    /// 2. The double of a point.
    ///
    /// The test cases are loaded from the [`G2_CURVE_TEST_CASES`] constant.
    #[test]
    fn test_g2_curve() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;

        // Running tests from file
        const DEBUG_FREQUENCY: usize = 2;
        for (i, test) in G2_CURVE_TEST_CASES.tests.iter().enumerate() {
            // Input:
            let mut point_1 = test.point_1.to_projective_point(cs);
            let mut point_2 = test.point_2.to_projective_point(cs);

            let point_2_x = point_2.x.clone();
            let point_2_y = point_2.y.clone();

            // Expected:
            let mut expected_sum = test.expected.sum.to_projective_point(cs);
            let mut expected_point_1_double = test.expected.point_1_double.to_projective_point(cs);
            let mut expected_point_2_double = test.expected.point_2_double.to_projective_point(cs);

            // Actual:
            let mut sum = point_1.add_mixed(cs, &mut (point_2_x, point_2_y));
            let mut point_1_double = point_1.double(cs);
            let mut point_2_double = point_2.double(cs);

            // Asserting:
            assert_equal_g2_points(cs, &mut sum, &mut expected_sum);
            assert_equal_g2_points(cs, &mut point_1_double, &mut expected_point_1_double);
            assert_equal_g2_points(cs, &mut point_2_double, &mut expected_point_2_double);

            debug_success("G2", i, DEBUG_FREQUENCY);
        }
    }

    /// Tests the line function doubling step evaluation used in the pairing computation.
    ///
    /// The test cases are loaded from the [`LINE_FUNCTION_TEST_CASES`] constant.
    #[test]
    fn test_doubling_step() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;

        // Running tests from file
        for (i, test) in LINE_FUNCTION_TEST_CASES.tests.iter().enumerate() {
            // Input:
            let mut g2_point_1 = test.g2_point_1.to_projective_point(cs);
            let mut g2_point_2 = test.g2_point_2.to_projective_point(cs);
            let mut g1_point = test.g1_point.to_projective_point(cs);

            // Expected:
            let mut expected_point_1 = test.expected.doubling_1.point.to_projective_point(cs);
            let mut expected_c0_1 = test.expected.doubling_1.c0.to_fq2(cs);
            let mut expected_c3_1 = test.expected.doubling_1.c3.to_fq2(cs);
            let mut expected_c4_1 = test.expected.doubling_1.c4.to_fq2(cs);

            let mut expected_point_2 = test.expected.doubling_2.point.to_projective_point(cs);
            let mut expected_c0_2 = test.expected.doubling_2.c0.to_fq2(cs);
            let mut expected_c3_2 = test.expected.doubling_2.c3.to_fq2(cs);
            let mut expected_c4_2 = test.expected.doubling_2.c4.to_fq2(cs);

            // Actual:
            let doubling_1 =
                LineFunctionEvaluation::doubling_step(cs, &mut g2_point_1, &mut g1_point);
            let mut point_1 = doubling_1.point();
            let (mut c0_1, mut c3_1, mut c4_1) = doubling_1.c0c3c4();

            let doubling_2 =
                LineFunctionEvaluation::doubling_step(cs, &mut g2_point_2, &mut g1_point);
            let mut point_2 = doubling_2.point();
            let (mut c0_2, mut c3_2, mut c4_2) = doubling_2.c0c3c4();

            // Asserting:
            assert_equal_g2_jacobian_points(cs, &mut point_1, &mut expected_point_1);
            assert_equal_fq2(cs, &mut c0_1, &mut expected_c0_1);
            assert_equal_fq2(cs, &mut c3_1, &mut expected_c3_1);
            assert_equal_fq2(cs, &mut c4_1, &mut expected_c4_1);

            assert_equal_g2_jacobian_points(cs, &mut point_2, &mut expected_point_2);
            assert_equal_fq2(cs, &mut c0_2, &mut expected_c0_2);
            assert_equal_fq2(cs, &mut c3_2, &mut expected_c3_2);
            assert_equal_fq2(cs, &mut c4_2, &mut expected_c4_2);

            println!("Line function test {} has passed!", i);
        }
    }

    /// Tests the line function addition step evaluation used in the pairing computation.
    ///
    /// The test cases are loaded from the [`LINE_FUNCTION_TEST_CASES`] constant.
    #[test]
    fn test_addition_step() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;

        // Running tests from file
        for (i, test) in LINE_FUNCTION_TEST_CASES.tests.iter().enumerate() {
            // Input:
            let mut g2_point_1 = test.g2_point_1.to_projective_point(cs);
            let mut g2_point_2 = test.g2_point_2.to_projective_point(cs);
            let mut g1_point = test.g1_point.to_projective_point(cs);

            // Expected:
            let mut expected_addition_point = test.expected.addition.point.to_projective_point(cs);
            let mut expected_c0 = test.expected.addition.c0.to_fq2(cs);
            let mut expected_c3 = test.expected.addition.c3.to_fq2(cs);
            let mut expected_c4 = test.expected.addition.c4.to_fq2(cs);

            // Actual:
            let addition = LineFunctionEvaluation::addition_step(
                cs,
                &mut g2_point_1,
                &mut g2_point_2,
                &mut g1_point,
            );
            let mut addition_point = addition.point();
            let (mut c0, mut c3, mut c4) = addition.c0c3c4();

            // Asserting:
            assert_equal_g2_jacobian_points(cs, &mut addition_point, &mut expected_addition_point);
            assert_equal_fq2(cs, &mut c0, &mut expected_c0);
            assert_equal_fq2(cs, &mut c3, &mut expected_c3);
            assert_equal_fq2(cs, &mut c4, &mut expected_c4);

            println!("Addition step function test {} has passed!", i);
        }
    }

    /// Tests the correctness of the following line operation inside the Miller Loop:
    /// - Double the first point
    /// - Add the second point
    ///
    /// The test cases are loaded from the [`LINE_FUNCTION_TEST_CASES`] constant.
    #[test]
    fn test_double_and_addition_step() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;

        // Running tests from file
        for (i, test) in LINE_FUNCTION_TEST_CASES.tests.iter().enumerate() {
            // Input:
            let mut g2_point_1 = test.g2_point_1.to_projective_point(cs);
            let mut g2_point_2 = test.g2_point_2.to_projective_point(cs);
            let mut g1_point = test.g1_point.to_projective_point(cs);

            // Expected:
            let mut expected_point = test
                .expected
                .doubling_1_and_addition
                .point
                .to_projective_point(cs);
            let mut expected_c0 = test.expected.doubling_1_and_addition.c0.to_fq2(cs);
            let mut expected_c3 = test.expected.doubling_1_and_addition.c3.to_fq2(cs);
            let mut expected_c4 = test.expected.doubling_1_and_addition.c4.to_fq2(cs);

            // Actual:
            let doubling =
                LineFunctionEvaluation::doubling_step(cs, &mut g2_point_1, &mut g1_point);
            g2_point_1 = doubling.point();
            let addition = LineFunctionEvaluation::addition_step(
                cs,
                &mut g2_point_2,
                &mut g2_point_1,
                &mut g1_point,
            );
            let mut actual_point = addition.point();
            let (mut c0, mut c3, mut c4) = addition.c0c3c4();

            // Asserting:
            assert_equal_g2_jacobian_points(cs, &mut actual_point, &mut expected_point);
            assert_equal_fq2(cs, &mut c0, &mut expected_c0);
            assert_equal_fq2(cs, &mut c3, &mut expected_c3);
            assert_equal_fq2(cs, &mut c4, &mut expected_c4);

            println!("Double&Addition step function test {} has passed!", i);
        }
    }

    /// Tests the Miller Loop step used in the pairing computation.
    ///
    /// The test cases are loaded from the [`PAIRING_TEST_CASES`] constant.
    #[test]
    #[ignore]
    fn test_miller_loop() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;

        // Running tests from file
        for (_, test) in PAIRING_TEST_CASES.tests.iter().enumerate() {
            // Input:
            let mut g1_point = test.g1_point.to_projective_point(cs);
            let mut g2_point = test.g2_point.to_projective_point(cs);

            // Expected:
            let mut expected_miller_loop = test.miller_loop.to_fq12(cs);

            // Actual:
            let miller_loop = MillerLoopEvaluation::evaluate(cs, &mut g1_point, &mut g2_point);
            let mut miller_loop = miller_loop.get_accumulated_f();

            // Asserting
            assert_equal_fq12(cs, &mut miller_loop, &mut expected_miller_loop);

            println!("Miller loop test has passed!");
        }
    }

    /// Tests the final exponentiation step used in the pairing computation.
    ///
    /// The test cases are loaded from the [`FINAL_EXP_TEST_CASES`] constant.
    #[test]
    #[ignore]
    fn test_final_exponentiation() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 25);
        let cs = &mut owned_cs;

        // Running tests from file
        for (i, test) in FINAL_EXP_TEST_CASES.tests.iter().enumerate() {
            // Expected:
            let expected_f_final = test.expected.to_fq12(cs);

            // Actual:
            let mut f = test.scalar.to_fq12(cs);
            let f_final = FinalExpEvaluation::evaluate(cs, &mut f);
            let f_final = f_final.get();

            assert_equal_fq12(cs, &f_final, &expected_f_final);

            println!("Final exponentiation test {} has passed!", i);
        }
    }

    /// Tests the EC pairing as a whole.
    ///
    /// The test cases are loaded from the [`PAIRING_TEST_CASES`] constant.
    #[test]
    #[ignore]
    fn test_ec_pairing() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;

        // Running tests from file
        for (i, test) in PAIRING_TEST_CASES.tests.iter().enumerate() {
            // Input:
            let mut g1_point = test.g1_point.to_projective_point(cs);
            let mut g2_point = test.g2_point.to_projective_point(cs);

            // Expected:
            let mut expected_pairing = test.pairing.to_fq12(cs);

            // Actual:
            let mut pairing = ec_pairing(cs, &mut g1_point, &mut g2_point);

            // Asserting
            assert_equal_fq12(cs, &mut pairing, &mut expected_pairing);

            println!("EC pairing test {} has passed!", i);
        }
    }
}
