pub mod test {
    use std::env::current_dir;
    use std::fs::File;
    use std::io::Read;

    use boojum::gadgets::traits::witnessable::WitnessHookable;
    use boojum::gadgets::{boolean::Boolean, curves::bn256::ec_add::projective_add};
    use boojum::pairing::ff::PrimeField;
    use boojum::pairing::CurveAffine;
    use lazy_static::lazy_static;
    use serde::{Deserialize, Serialize};

    use crate::bn254::tests::json::EC_ADD_TEST_CASES;
    use crate::bn254::tests::utils::{create_test_cs, RawPoint};
    use crate::bn254::{BN256Affine, BN256Fq, BN256Fr};

    #[test]
    fn test_addition() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;
        let boolean_false = Boolean::allocated_constant(cs, false);

        for (i, test) in EC_ADD_TEST_CASES.tests.iter().enumerate() {
            let mut point_1 = test.point_1.to_projective_point(cs);
            let point_2 = test.point_2.to_projective_point(cs);

            let mut sum = projective_add(cs, &mut point_1, point_2);
            let (sum, at_infty) = sum.convert_to_affine_or_default(cs, BN256Affine::one());

            Boolean::enforce_equal(cs, &at_infty, &boolean_false);

            let x_actual = sum.0.witness_hook(cs)().unwrap().get();
            let y_actual = sum.1.witness_hook(cs)().unwrap().get();

            let expected_result = &test.expected;
            let x_expected = BN256Fq::from_str(expected_result.x.as_str()).unwrap();
            let y_expected = BN256Fq::from_str(expected_result.y.as_str()).unwrap();

            assert_eq!(x_expected, x_actual, "Test case {} failed: x coordinates are not equal", i+1);
            assert_eq!(y_expected, y_actual, "Test case {} failed: y coordinates are not equal", i+1);

            // Print a message every 10 tests
            if i % 10 == 9 {
                println!("EC addition tests {} to {} have passed", i-8, i+1);
            }
        }
    }
}
