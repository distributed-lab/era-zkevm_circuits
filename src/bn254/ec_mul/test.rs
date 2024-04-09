pub mod test {
    use crate::bn254::fixed_base_mul_table::{create_fixed_base_mul_table, FixedBaseMulTable};
    use crate::bn254::{
        BN256Affine, BN256BaseNNField, BN256BaseNNFieldParams, BN256Fr, BN256Fq, BN256ScalarNNField,
        BN256ScalarNNFieldParams,
    };
    use boojum::gadgets::curves::bn256::ec_mul::{width_4_windowed_multiplication, ScalarDecomposition};
    use boojum::gadgets::curves::sw_projective::SWProjectivePoint;
    use boojum::gadgets::non_native_field::implementations::NonNativeFieldOverU16Params;
    use boojum::gadgets::non_native_field::traits::NonNativeField;
    use boojum::gadgets::traits::witnessable::WitnessHookable;
    use boojum::pairing::ff::{Field, PrimeField};
    use boojum::pairing::{CurveAffine, CurveProjective};
    use boojum::{
        config::DevCSConfig,
        cs::{
            cs_builder::{new_builder, CsBuilder, CsBuilderImpl},
            cs_builder_reference::CsReferenceImplementationBuilder,
            gates::{
                BooleanConstraintGate, ConstantsAllocatorGate, DotProductGate,
                FmaGateInBaseFieldWithoutConstant, NopGate, ReductionGate, SelectionGate,
                U8x4FMAGate, UIntXAddGate, ZeroCheckGate,
            },
            implementations::reference_cs::CSReferenceImplementation,
            traits::{cs::ConstraintSystem, gate::GatePlacementStrategy},
            CSGeometry, GateConfigurationHolder, LookupParameters, StaticToolboxHolder,
        },
        field::{goldilocks::GoldilocksField, SmallField},
        gadgets::tables::{
            create_and8_table, create_byte_split_table, create_xor8_table, And8Table,
            ByteSplitTable, Xor8Table,
        },
    };
    use std::sync::Arc;

    type F = GoldilocksField;
    type P = GoldilocksField;

    /// Creates a test constraint system for testing purposes
    fn create_test_cs(
        max_trace_len: usize,
    ) -> CSReferenceImplementation<
        F,
        P,
        DevCSConfig,
        impl GateConfigurationHolder<F>,
        impl StaticToolboxHolder,
    > {
        let geometry = CSGeometry {
            num_columns_under_copy_permutation: 100,
            num_witness_columns: 0,
            num_constant_columns: 8,
            max_allowed_constraint_degree: 4,
        };
        let max_variables = 1 << 26;

        fn configure<
            F: SmallField,
            T: CsBuilderImpl<F, T>,
            GC: GateConfigurationHolder<F>,
            TB: StaticToolboxHolder,
        >(
            builder: CsBuilder<T, F, GC, TB>,
        ) -> CsBuilder<T, F, impl GateConfigurationHolder<F>, impl StaticToolboxHolder> {
            let builder = builder.allow_lookup(
                LookupParameters::UseSpecializedColumnsWithTableIdAsConstant {
                    width: 3,
                    num_repetitions: 8,
                    share_table_id: true,
                },
            );
            let builder = U8x4FMAGate::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = ConstantsAllocatorGate::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = FmaGateInBaseFieldWithoutConstant::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = ReductionGate::<F, 4>::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            // let owned_cs = ReductionGate::<F, 4>::configure_for_cs(owned_cs, GatePlacementStrategy::UseSpecializedColumns { num_repetitions: 8, share_constants: true });
            let builder = BooleanConstraintGate::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = UIntXAddGate::<32>::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = UIntXAddGate::<16>::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = UIntXAddGate::<8>::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = SelectionGate::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            let builder = ZeroCheckGate::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
                false,
            );
            let builder = DotProductGate::<4>::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );
            // let owned_cs = DotProductGate::<4>::configure_for_cs(owned_cs, GatePlacementStrategy::UseSpecializedColumns { num_repetitions: 1, share_constants: true });
            let builder = NopGate::configure_builder(
                builder,
                GatePlacementStrategy::UseGeneralPurposeColumns,
            );

            builder
        }

        let builder_impl =
            CsReferenceImplementationBuilder::<F, P, DevCSConfig>::new(geometry, max_trace_len);
        let builder = new_builder::<_, F>(builder_impl);

        let builder = configure(builder);
        let mut owned_cs = builder.build(max_variables);

        // add tables
        let table = create_xor8_table();
        owned_cs.add_lookup_table::<Xor8Table, 3>(table);

        let table = create_and8_table();
        owned_cs.add_lookup_table::<And8Table, 3>(table);

        seq_macro::seq!(C in 0..32 {
            let table = create_fixed_base_mul_table::<F, 0, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<0, C>, 3>(table);
            let table = create_fixed_base_mul_table::<F, 1, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<1, C>, 3>(table);
            let table = create_fixed_base_mul_table::<F, 2, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<2, C>, 3>(table);
            let table = create_fixed_base_mul_table::<F, 3, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<3, C>, 3>(table);
            let table = create_fixed_base_mul_table::<F, 4, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<4, C>, 3>(table);
            let table = create_fixed_base_mul_table::<F, 5, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<5, C>, 3>(table);
            let table = create_fixed_base_mul_table::<F, 6, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<6, C>, 3>(table);
            let table = create_fixed_base_mul_table::<F, 7, C>();
            owned_cs.add_lookup_table::<FixedBaseMulTable<7, C>, 3>(table);
        });

        let table = create_byte_split_table::<F, 1>();
        owned_cs.add_lookup_table::<ByteSplitTable<1>, 3>(table);
        let table = create_byte_split_table::<F, 2>();
        owned_cs.add_lookup_table::<ByteSplitTable<2>, 3>(table);
        let table = create_byte_split_table::<F, 3>();
        owned_cs.add_lookup_table::<ByteSplitTable<3>, 3>(table);
        let table = create_byte_split_table::<F, 4>();
        owned_cs.add_lookup_table::<ByteSplitTable<4>, 3>(table);

        owned_cs
    }

    fn bn254_base_field_params() -> BN256BaseNNFieldParams {
        NonNativeFieldOverU16Params::create()
    }

    fn bn254_scalar_field_params() -> BN256ScalarNNFieldParams {
        NonNativeFieldOverU16Params::create()
    }

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

        let test_scalar = BN256Fr::from_str("9997758448649743481679332046642653083029331058711609633943349318238462807072").unwrap();
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
        let mut seed_base  = BN256Fr::multiplicative_generator().pow([2222]);

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

            let mut actual =
                width_4_windowed_multiplication(cs, point_nn, scalar_nn, &base_params, &scalar_params);
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
