pub mod test {
    use std::env::current_dir;
    use std::fs::File;
    use std::io::Read;

    use boojum::gadgets::traits::witnessable::WitnessHookable;
    use boojum::gadgets::boolean::Boolean;
    use boojum::pairing::ff::PrimeField;
    use boojum::pairing::CurveAffine;
    use lazy_static::lazy_static;
    use serde::{Deserialize, Serialize};

    use crate::bn254::ec_add::implementation::projective_add;
    use crate::bn254::tests::json::types::RawG1Point;
    use crate::bn254::tests::json::EC_ADD_TEST_CASES;
    use crate::bn254::tests::utils::assert::assert_equal_g1_points;
    use crate::bn254::tests::utils::cs::create_test_cs;
    use crate::bn254::tests::utils::debug_success;
    use crate::bn254::{BN256Affine, BN256Fq, BN256Fr};

    #[test]
    fn test_addition() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;

        // Runnings tests from file
        const DEBUG_FREQUENCY: usize = 10;
        for (i, test) in EC_ADD_TEST_CASES.tests.iter().enumerate() {
            // Expected:
            let mut expected_sum = test.expected.to_projective_point(cs);

            // Actual:
            let mut point_1 = test.point_1.to_projective_point(cs);
            let (x, y) = test.point_2.to_coordinates(cs);
            let mut sum = projective_add(cs, &mut point_1,(x, y));

            assert_equal_g1_points(cs, &mut sum, &mut expected_sum);

            debug_success("ec_add", i, DEBUG_FREQUENCY);
        }
    }
}
