pub mod test {
    use boojum::gadgets::traits::witnessable::WitnessHookable;
    use boojum::gadgets::{boolean::Boolean, curves::bn256::ec_add::projective_add};
    use boojum::pairing::ff::PrimeField;
    use boojum::pairing::CurveAffine;

    use crate::bn254::test_utils::{create_test_cs, RawPoint};
    use crate::bn254::{BN256Affine, BN256Fq, BN256Fr};

    struct AddTestCase {
        point_1: RawPoint,
        point_2: RawPoint,
        expected_result: RawPoint,
    }

    #[test]
    fn test_addition() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;
        let boolean_false = Boolean::allocated_constant(cs, false);

        let tests = vec![AddTestCase {
            point_1: RawPoint {
                x: "10427591839758008194904206209576776002642396244985815734642327872507511451679"
                    .to_string(),
                y: "7636615425298746878120246585210560605147292449856688181998147483645903757740"
                    .to_string(),
            },
            point_2: RawPoint {
                x: "5079615400120625967577872611520829322936839187315105706783827450466730000282"
                    .to_string(),
                y: "10130758747227163499602368437255336991898808119637262386824177410977966112071"
                    .to_string(),
            },
            expected_result: RawPoint {
                x: "13830304928143253030817203464067949762009877639643889333966531725855231388805"
                    .to_string(),
                y: "17746262971018357685371317122061874723598233511610223591578743849381939828484"
                    .to_string(),
            },
        }];

        for (i, test) in tests.iter().enumerate() {
            let mut point_1 = test.point_1.to_projective_point(cs);
            let point_2 = test.point_2.to_projective_point(cs);

            let mut sum = projective_add(cs, &mut point_1, point_2);
            let (sum, at_infty) = sum.convert_to_affine_or_default(cs, BN256Affine::one());

            Boolean::enforce_equal(cs, &at_infty, &boolean_false);

            let x_actual = sum.0.witness_hook(cs)().unwrap().get();
            let y_actual = sum.1.witness_hook(cs)().unwrap().get();

            let expected_result = &test.expected_result;
            let x_expected = BN256Fq::from_str(expected_result.x.as_str()).unwrap();
            let y_expected = BN256Fq::from_str(expected_result.y.as_str()).unwrap();

            assert_eq!(x_expected, x_actual);
            assert_eq!(y_expected, y_actual);

            println!("Addition test {} passed", i + 1);
        }
    }
}
