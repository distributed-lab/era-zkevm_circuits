pub mod test {
    use std::fs::File;
    use std::io::Read;
    use std::sync::Arc;

    use boojum::gadgets::tower_extension::algebraic_torus::TorusWrapper;
    use boojum::gadgets::tower_extension::params::bn256::BN256Extension12Params;
    use lazy_static::lazy_static;
    use serde::{Deserialize, Serialize};

    use boojum::cs::traits::cs::ConstraintSystem;
    use boojum::field::goldilocks::GoldilocksField;
    use boojum::gadgets::curves::sw_projective::SWProjectivePoint;
    use boojum::gadgets::traits::witnessable::WitnessHookable;
    use boojum::pairing::ff::{Field, PrimeField};
    use boojum::pairing::{CurveAffine, CurveProjective};

    use crate::bn254::tests::json::{
        DECOMPOSITION_TEST_CASES, EC_MUL_TEST_CASES, FQ12_TEST_CASES, FQ2_TEST_CASES,
        FQ6_TEST_CASES, TORUS_TEST_CASES,
    };
    use crate::bn254::tests::utils::assert::{
        assert_equal_fq12, assert_equal_fq2, assert_equal_fq6,
    };
    use crate::bn254::tests::utils::cs::{
        bn254_base_field_params, bn254_scalar_field_params, create_test_cs,
    };
    use crate::bn254::tests::utils::debug_success;
    use crate::bn254::{BN256Affine, BN256BaseNNField, BN256Fq, BN256Fr, BN256ScalarNNField, BN256TorusWrapper};

    type F = GoldilocksField;
    type P = GoldilocksField;

    /// Test the compression and decompression functions in Algebraic Torus.
    ///
    /// The tests are run against the test cases defined in [`TORUS_TEST_CASES`], which
    /// are generated using the `sage` script in `gen/torus.sage`.
    #[test]
    fn test_torus_compression() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;

        // Running tests from file: validating correctness of encoded values
        const DEBUG_FREQUENCY: usize = 2;
        for (i, test) in TORUS_TEST_CASES.tests[..1].iter().enumerate() {
            // Reading inputs
            let mut scalar_1 = test.scalar_1.to_fq12(cs);
            let mut scalar_2 = test.scalar_2.to_fq12(cs);

            // Actual (compressing inputs):
            let scalar_1_torus: BN256TorusWrapper<F> = TorusWrapper::compress::<_, true>(cs, &mut scalar_1);
            let scalar_2_torus: BN256TorusWrapper<F> = TorusWrapper::compress::<_, true>(cs, &mut scalar_2);
            
            // Expected:
            let expected_encoding_1 = test.expected.encoding_1.to_fq6(cs);
            let expected_encoding_2 = test.expected.encoding_2.to_fq6(cs);

            // Asserting:
            assert_equal_fq6(cs, &scalar_1_torus.encoding, &expected_encoding_1);
            assert_equal_fq6(cs, &scalar_2_torus.encoding, &expected_encoding_2);

            debug_success("torus compression", i, DEBUG_FREQUENCY);
        }
    }

    /// Test basic arithmetic on Algebraic Torus.
    ///
    /// - multiplication (`.mul`)
    /// - inverse (`.inverse`)
    /// - conjugate (`.conjugate`)
    /// - squaring (`.square`)
    ///
    /// The tests are run against the test cases defined in [`TORUS_TEST_CASES`], which
    /// are generated using the `sage` script in `gen/torus.sage`.
    #[test]
    fn test_torus_basic_arithmetic() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;

        // Running tests from file: validating sum, diff, prod, and quot
        const DEBUG_FREQUENCY: usize = 2;
        for (i, test) in TORUS_TEST_CASES.tests.iter().enumerate() {
            // Reading inputs
            let mut scalar_1 = test.scalar_1.to_fq12(cs);
            let mut scalar_2 = test.scalar_2.to_fq12(cs);

            // Compressing inputs
            let mut scalar_1_torus: BN256TorusWrapper<F> = TorusWrapper::compress::<_, true>(cs, &mut scalar_1);
            let mut scalar_2_torus: BN256TorusWrapper<F> = TorusWrapper::compress::<_, true>(cs, &mut scalar_2);
            // Expected:
            let expected_product = test.expected.product_encoding.to_fq6(cs);
            let expected_inverse_1 = test.expected.inverse_1_encoding.to_fq6(cs);
            let expected_conjugate_1 = test.expected.conjugate_1_encoding.to_fq6(cs);
            let expected_square_1 = test.expected.square_1_encoding.to_fq6(cs);

            // Actual:
            let product = scalar_1_torus.mul::<_, true>(cs, &mut scalar_2_torus);
            let inverse_1 = scalar_1_torus.inverse(cs);
            let conjugate_1 = scalar_1_torus.conjugate(cs);
            let square_1 = scalar_1_torus.square::<_, true>(cs);

            // Asserting:
            assert_equal_fq6(cs, &product.encoding, &expected_product);
            assert_equal_fq6(cs, &inverse_1.encoding, &expected_inverse_1);
            assert_equal_fq6(cs, &conjugate_1.encoding, &expected_conjugate_1);
            assert_equal_fq6(cs, &square_1.encoding, &expected_square_1);

            debug_success("torus basic arithmetic", i, DEBUG_FREQUENCY);
        }
    }
}
