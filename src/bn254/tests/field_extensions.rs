pub mod test {
    use std::fs::File;
    use std::io::Read;
    use std::sync::Arc;

    use boojum::cs::traits::cs::ConstraintSystem;
    use lazy_static::lazy_static;
    use serde::{Deserialize, Serialize};

    use boojum::field::goldilocks::GoldilocksField;
    use boojum::gadgets::curves::bn256::ec_mul::{
        width_4_windowed_multiplication, ScalarDecomposition,
    };
    use boojum::gadgets::curves::bn256::{BN256BaseNNField, BN256Fq12NNField, BN256Fq2NNField, BN256ScalarNNField};
    use boojum::gadgets::curves::sw_projective::SWProjectivePoint;
    use boojum::gadgets::traits::witnessable::WitnessHookable;
    use boojum::pairing::ff::{Field, PrimeField};
    use boojum::pairing::{CurveAffine, CurveProjective};

    use crate::bn254::tests::json::{
        DECOMPOSITION_TEST_CASES, EC_MUL_TEST_CASES, FQ12_TEST_CASES, FQ2_TEST_CASES, FQ6_TEST_CASES
    };
    use crate::bn254::tests::types::{
        bn254_base_field_params, bn254_scalar_field_params, create_test_cs,
    };
    use crate::bn254::{BN256Affine, BN256Fq, BN256Fq6NNField, BN256Fr};

    type F = GoldilocksField;
    type P = GoldilocksField;

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

    #[test]
    fn test_fq2_operations() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;

        // Running tests from file: validating sum, diff, prod, and quot
        for (i, test) in FQ2_TEST_CASES.tests.iter().enumerate() {
            let mut scalar_1 = test.scalar_1.to_fq2(cs);
            let mut scalar_2 = test.scalar_2.to_fq2(cs);

            let expected_sum = test.expected.sum.to_fq2(cs);
            let expected_difference = test.expected.difference.to_fq2(cs);
            let expected_product = test.expected.product.to_fq2(cs);
            let expected_quotient = test.expected.quotient.to_fq2(cs);
            let expected_scalar_1_nonresidue = test.expected.scalar_1_non_residue.to_fq2(cs);

            let sum = scalar_1.add(cs, &mut scalar_2);
            let difference = scalar_1.sub(cs, &mut scalar_2);
            let product = scalar_1.mul(cs, &mut scalar_2);
            let quotient = scalar_1.div(cs, &mut scalar_2);
            let scalar_1_non_residue = scalar_1.mul_by_nonresidue(cs);

            assert_equal_fq2(cs, &sum, &expected_sum, "Sum test failed");
            assert_equal_fq2(
                cs,
                &difference,
                &expected_difference,
                "Difference test failed",
            );
            assert_equal_fq2(cs, &product, &expected_product, "Product test failed");
            assert_equal_fq2(cs, &quotient, &expected_quotient, "Quotient test failed");
            assert_equal_fq2(
                cs,
                &scalar_1_non_residue,
                &expected_scalar_1_nonresidue,
                "Scalar 1 non-residue test failed",
            );

            // Print a message every 10 tests
            if i % 10 == 9 {
                println!("Decomposition tests {} to {} have passed", i - 8, i + 1);
            }
        }
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

    #[test]
    fn test_fq6_operations() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;

        // Running tests from file: validating sum, diff, prod, and quot
        for (i, test) in FQ6_TEST_CASES.tests.iter().enumerate() {
            let mut scalar_1 = test.scalar_1.to_fq6(cs);
            let mut scalar_2 = test.scalar_2.to_fq6(cs);
            let mut c0 = test.c0.to_fq2(cs);
            let mut c1 = test.c1.to_fq2(cs);

            let expected_sum = test.expected.sum.to_fq6(cs);
            let expected_difference = test.expected.difference.to_fq6(cs);
            let expected_product = test.expected.product.to_fq6(cs);
            let expected_quotient = test.expected.quotient.to_fq6(cs);
            let expected_product_c1 = test.expected.product_c1.to_fq6(cs);
            let expected_product_c0c1 = test.expected.product_c0c1.to_fq6(cs);
            let expected_scalar_1_inverse = test.expected.scalar_1_inverse.to_fq6(cs);
            let expected_scalar_1_square = test.expected.scalar_1_square.to_fq6(cs);
            let expected_scalar_1_non_residue = test.expected.scalar_1_non_residue.to_fq6(cs);

            let sum = scalar_1.add(cs, &mut scalar_2);
            let difference = scalar_1.sub(cs, &mut scalar_2);
            let product = scalar_1.mul(cs, &mut scalar_2);
            let quotient = scalar_1.div(cs, &mut scalar_2);
            let product_c1 = scalar_1.mul_by_c1(cs, &mut c1);
            let product_c0c1 = scalar_1.mul_by_c0c1(cs, &mut c0, &mut c1);
            let scalar_1_inverse = scalar_1.inverse(cs);
            let scalar_1_square = scalar_1.square(cs);
            let scalar_1_non_residue = scalar_1.mul_by_nonresidue(cs);

            assert_equal_fq6(cs, &sum, &expected_sum, "Sum test failed");
            assert_equal_fq6(
                cs,
                &difference,
                &expected_difference,
                "Difference test failed",
            );
            assert_equal_fq6(cs, &product, &expected_product, "Product test failed");
            assert_equal_fq6(cs, &quotient, &expected_quotient, "Quotient test failed");
            assert_equal_fq6(
                cs,
                &product_c1,
                &expected_product_c1,
                "Product c1 test failed",
            );
            assert_equal_fq6(
                cs,
                &product_c0c1,
                &expected_product_c0c1,
                "Product c0c1 test failed",
            );
            assert_equal_fq6(
                cs,
                &scalar_1_inverse,
                &expected_scalar_1_inverse,
                "Scalar 1 inverse test failed",
            );
            assert_equal_fq6(
                cs,
                &scalar_1_square,
                &expected_scalar_1_square,
                "Scalar 1 square test failed",
            );
            assert_equal_fq6(
                cs,
                &scalar_1_non_residue,
                &expected_scalar_1_non_residue,
                "Scalar 1 non-residue test failed",
            );

            // Print a message every 10 tests
            if i % 10 == 9 {
                println!("Decomposition tests {} to {} have passed", i - 8, i + 1);
            }
        }
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
    fn test_fq12_operations() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;

        // Running tests from file: validating sum, diff, prod, and quot
        for (i, test) in FQ12_TEST_CASES.tests.iter().enumerate() {
            // Reading inputs
            let mut scalar_1 = test.scalar_1.to_fq12(cs);
            let mut scalar_2 = test.scalar_2.to_fq12(cs);
            let mut c0 = test.c0.to_fq2(cs);
            let mut c1 = test.c1.to_fq2(cs);
            let mut c3 = test.c3.to_fq2(cs);
            let mut c4 = test.c4.to_fq2(cs);

            // Getting expected outputs
            let expected_sum = test.expected.sum.to_fq12(cs);
            let expected_difference = test.expected.difference.to_fq12(cs);
            let expected_product = test.expected.product.to_fq12(cs);
            let expected_product_c0c3c4 = test.expected.product_c0c3c4.to_fq12(cs);
            let expected_product_c0c1c4 = test.expected.product_c0c1c4.to_fq12(cs);
            let expected_quotient = test.expected.quotient.to_fq12(cs);
            let expected_scalar_1_inverse = test.expected.scalar_1_inverse.to_fq12(cs);
            let expected_scalar_1_square = test.expected.scalar_1_square.to_fq12(cs);

            // Finding actual results of operations
            let sum = scalar_1.add(cs, &mut scalar_2);
            let difference = scalar_1.sub(cs, &mut scalar_2);
            let product = scalar_1.mul(cs, &mut scalar_2);
            let product_c0c3c4 = scalar_1.mul_by_c0c3c4(cs, &mut c0, &mut c3, &mut c4);
            let product_c0c1c4 = scalar_1.mul_by_c0c1c4(cs, &mut c0, &mut c1, &mut c4);
            let quotient = scalar_1.div(cs, &mut scalar_2);
            let scalar_1_inverse = scalar_1.inverse(cs);
            let scalar_1_square = scalar_1.square(cs);
        
            assert_equal_fq12(cs, &sum, &expected_sum, "Sum test failed");
            assert_equal_fq12(
                cs,
                &difference,
                &expected_difference,
                "Difference test failed",
            );
            assert_equal_fq12(cs, &product, &expected_product, "Product test failed");
            assert_equal_fq12(
                cs,
                &product_c0c3c4,
                &expected_product_c0c3c4,
                "Product c0c3c4 test failed",
            );
            assert_equal_fq12(
                cs,
                &product_c0c1c4,
                &expected_product_c0c1c4,
                "Product c0c1c4 test failed",
            );
            assert_equal_fq12(cs, &quotient, &expected_quotient, "Quotient test failed");
            assert_equal_fq12(
                cs,
                &scalar_1_inverse,
                &expected_scalar_1_inverse,
                "Scalar 1 inverse test failed",
            );
            assert_equal_fq12(
                cs,
                &scalar_1_square,
                &expected_scalar_1_square,
                "Scalar 1 square test failed",
            );

            // Print a message every 10 tests
            if i % 10 == 9 {
                println!("Decomposition tests {} to {} have passed", i - 8, i + 1);
            }
        }
    }
}
