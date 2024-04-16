// Helper utils for testing

use std::sync::Arc;

use boojum::{
    config::DevCSConfig,
    cs::{
        cs_builder::{new_builder, CsBuilder, CsBuilderImpl},
        cs_builder_reference::CsReferenceImplementationBuilder,
        gates::{
            BooleanConstraintGate, ConstantsAllocatorGate, DotProductGate,
            FmaGateInBaseFieldWithoutConstant, NopGate, ReductionGate, SelectionGate, U8x4FMAGate,
            UIntXAddGate, ZeroCheckGate,
        },
        implementations::reference_cs::CSReferenceImplementation,
        traits::{cs::ConstraintSystem, gate::GatePlacementStrategy},
        CSGeometry, GateConfigurationHolder, LookupParameters, StaticToolboxHolder,
    },
    field::{goldilocks::GoldilocksField, SmallField},
    gadgets::{
        curves::bn256::{
            BN256BaseNNField, BN256BaseNNFieldParams, BN256Fq12NNField, BN256Fq2NNField, BN256SWProjectivePoint, BN256SWProjectivePointTwisted, BN256ScalarNNFieldParams
        },
        non_native_field::implementations::NonNativeFieldOverU16Params,
        tables::{
            create_and8_table, create_byte_split_table, create_xor8_table, And8Table,
            ByteSplitTable, Xor8Table,
        },
    },
    pairing::ff::PrimeField,
};
use serde::{Deserialize, Serialize};

use crate::bn254::{BN256Fq, BN256Fq6NNField, BN256Fr};

use crate::bn254::fixed_base_mul_table::{create_fixed_base_mul_table, FixedBaseMulTable};

type F = GoldilocksField;
type P = GoldilocksField;

/// Creates a test constraint system for testing purposes
pub fn create_test_cs(
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
        let builder =
            NopGate::configure_builder(builder, GatePlacementStrategy::UseGeneralPurposeColumns);

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

/// Returns BN254 base field parameters
pub fn bn254_base_field_params() -> BN256BaseNNFieldParams {
    NonNativeFieldOverU16Params::create()
}

/// Returns BN254 scalar field parameters
pub fn bn254_scalar_field_params() -> BN256ScalarNNFieldParams {
    NonNativeFieldOverU16Params::create()
}

/// Representation of an elliptic curve point in raw form (as strings)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawG1Point {
    pub x: String,
    pub y: String,
}

impl RawG1Point {
    /// Converts a raw point to a projective point
    pub fn to_projective_point<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
    ) -> BN256SWProjectivePoint<F> {
        let base_params = Arc::new(bn254_base_field_params());

        let x = BN256Fq::from_str(self.x.as_str()).unwrap();
        let y = BN256Fq::from_str(self.y.as_str()).unwrap();

        let x_nn = BN256BaseNNField::allocate_checked(cs, x, &base_params);
        let y_nn = BN256BaseNNField::allocate_checked(cs, y, &base_params);

        BN256SWProjectivePoint::<F>::from_xy_unchecked(cs, x_nn, y_nn)
    }
}

/// Representation of a G2 elliptic curve point in raw form (as strings)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawG2Point {
    pub x: RawFq2,
    pub y: RawFq2,
}

impl RawG2Point {
    /// Converts a raw point to a projective point
    pub fn to_projective_point<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
    ) -> BN256SWProjectivePointTwisted<F> {
        let x_nn = self.x.to_fq2(cs);
        let y_nn = self.y.to_fq2(cs);

        BN256SWProjectivePointTwisted::<F>::from_xy_unchecked(cs, x_nn, y_nn)
    }
}

/// Representation of an `Fq2` element in a raw form (as strings)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawFq2 {
    pub c0: String,
    pub c1: String,
}

impl RawFq2 {
    /// Converts a raw point to a non-native fq2 element
    pub fn to_fq2<CS: ConstraintSystem<F>>(&self, cs: &mut CS) -> BN256Fq2NNField<F> {
        let base_params = Arc::new(bn254_base_field_params());

        let c0 = BN256Fq::from_str(self.c0.as_str()).unwrap();
        let c0 = BN256BaseNNField::allocate_checked(cs, c0, &base_params);

        let c1 = BN256Fq::from_str(self.c1.as_str()).unwrap();
        let c1 = BN256BaseNNField::allocate_checked(cs, c1, &base_params);

        BN256Fq2NNField::new(c0, c1)
    }
}

/// Representation of an `Fq6` element in a raw form (as strings)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawFq6 {
    pub c0: RawFq2,
    pub c1: RawFq2,
    pub c2: RawFq2,
}

impl RawFq6 {
    /// Converts a raw point to a non-native `Fq6` element
    pub fn to_fq6<CS: ConstraintSystem<F>>(&self, cs: &mut CS) -> BN256Fq6NNField<F> {
        let c0 = self.c0.to_fq2(cs);
        let c1 = self.c1.to_fq2(cs);
        let c2 = self.c2.to_fq2(cs);

        BN256Fq6NNField::new(c0, c1, c2)
    }
}

/// Representation of an `Fq12` element in a raw form (as strings)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawFq12 {
    pub c0: RawFq6,
    pub c1: RawFq6,
}

impl RawFq12 {
    /// Converts a raw point to a non-native `Fq12` element
    pub fn to_fq12<CS: ConstraintSystem<F>>(&self, cs: &mut CS) -> BN256Fq12NNField<F> {
        let c0 = self.c0.to_fq6(cs);
        let c1 = self.c1.to_fq6(cs);

        BN256Fq12NNField::new(c0, c1)
    }
}
