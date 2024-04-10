pub mod test {
    use crate::bn254::ec_add::{self, bn254_base_field_params, bn254_scalar_field_params};
    use crate::bn254::fixed_base_mul_table::{create_fixed_base_mul_table, FixedBaseMulTable};
    use crate::bn254::{
        BN256Affine, BN256BaseNNField, BN256BaseNNFieldParams, BN256Fr, BN256Fq, BN256ScalarNNField,
        BN256ScalarNNFieldParams,
    };
    use boojum::gadgets::boolean::Boolean;
    use boojum::gadgets::curves::bn256::ec_add::projective_add;
    use boojum::gadgets::curves::bn256::ec_mul::{width_4_windowed_multiplication, ScalarDecomposition};
    use boojum::gadgets::curves::bn256::BN256SWProjectivePoint;
    use boojum::gadgets::curves::sw_projective::SWProjectivePoint;
    use boojum::gadgets::non_native_field::implementations::NonNativeFieldOverU16Params;
    use boojum::gadgets::non_native_field::traits::NonNativeField;
    use boojum::gadgets::traits::witnessable::WitnessHookable;
    use boojum::pairing::compact_bn256::G1Affine;
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

    #[test]
    fn test_addition() {
        // Preparing the constraint system and parameters
        let mut owned_cs = create_test_cs(1 << 21);
        let cs = &mut owned_cs;
        let base_params = Arc::new(bn254_base_field_params());

        let x1 = BN256Fq::from_str("10427591839758008194904206209576776002642396244985815734642327872507511451679").unwrap();
        let y1 = BN256Fq::from_str("7636615425298746878120246585210560605147292449856688181998147483645903757740").unwrap();

        let x2 = BN256Fq::from_str("5079615400120625967577872611520829322936839187315105706783827450466730000282").unwrap();
        let y2 = BN256Fq::from_str("10130758747227163499602368437255336991898808119637262386824177410977966112071").unwrap();

        let x12 = BN256Fq::from_str("13830304928143253030817203464067949762009877639643889333966531725855231388805").unwrap();
        let y12 = BN256Fq::from_str("17746262971018357685371317122061874723598233511610223591578743849381939828484").unwrap();

        let x1_nn = BN256BaseNNField::allocate_checked(cs, x1, &base_params);
        let y1_nn = BN256BaseNNField::allocate_checked(cs, y1, &base_params);
        let mut point1 = BN256SWProjectivePoint::<F>::from_xy_unchecked(cs, x1_nn, y1_nn);

        let x2_nn = BN256BaseNNField::allocate_checked(cs, x2, &base_params);
        let y2_nn = BN256BaseNNField::allocate_checked(cs, y2, &base_params);
        let point2 = BN256SWProjectivePoint::<F>::from_xy_unchecked(cs, x2_nn, y2_nn);

        let mut sum = projective_add(cs, &mut point1, point2);
        let (sum, at_infty) = sum.convert_to_affine_or_default(cs, BN256Affine::one());
        let boolean_false = Boolean::allocated_constant(cs, false);

        Boolean::enforce_equal(cs, &at_infty, &boolean_false);

        let x12_actual = sum.0.witness_hook(cs)().unwrap().get();
        let y12_actual = sum.1.witness_hook(cs)().unwrap().get();
        
        assert_eq!(x12, x12_actual);
        assert_eq!(y12, y12_actual);
    }
}
