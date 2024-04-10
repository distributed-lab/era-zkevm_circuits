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
    use boojum::gadgets::curves::bn256::{BN256BaseNNField, BN256Fq2NNField, BN256ScalarNNField};
    use boojum::gadgets::curves::sw_projective::SWProjectivePoint;
    use boojum::gadgets::traits::witnessable::WitnessHookable;
    use boojum::pairing::ff::{Field, PrimeField};
    use boojum::pairing::{CurveAffine, CurveProjective};

    use crate::bn254::tests::json::{DECOMPOSITION_TEST_CASES, EC_MUL_TEST_CASES, FQ2_TEST_CASES};
    use crate::bn254::tests::utils::{
        bn254_base_field_params, bn254_scalar_field_params, create_test_cs,
    };
    use crate::bn254::{BN256Affine, BN256Fq, BN256Fr};

    type F = GoldilocksField;
    type P = GoldilocksField;

    fn assert_equal_fq2<CS: ConstraintSystem<F>>(cs: &mut CS, a: &BN256Fq2NNField<F>, b: &BN256Fq2NNField<F>, msg: &str) {
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
            let expected_diff = test.expected.diff.to_fq2(cs);
            let expected_prod = test.expected.prod.to_fq2(cs);
            let expected_quot = test.expected.quot.to_fq2(cs);

            let sum = scalar_1.add(cs, &mut scalar_2);
            let diff = scalar_1.sub(cs, &mut scalar_2);
            let prod = scalar_1.mul(cs, &mut scalar_2);
            let quot = scalar_1.div(cs, &mut scalar_2);

            assert_equal_fq2(cs, &sum, &expected_sum, "Sum test failed");
            assert_equal_fq2(cs, &diff, &expected_diff, "Diff test failed");
            assert_equal_fq2(cs, &prod, &expected_prod, "Prod test failed");
            assert_equal_fq2(cs, &quot, &expected_quot, "Quot test failed");            

            // Print a message every 10 tests
            if i % 10 == 9 {
                println!("Decomposition tests {} to {} have passed", i - 8, i + 1);
            }
        }
    }
}
