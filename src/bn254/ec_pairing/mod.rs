use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use boojum::algebraic_props::round_function::AlgebraicRoundFunction;
use boojum::crypto_bigint::Zero;
use boojum::cs::gates::{ConstantAllocatableCS, PublicInputGate};
use boojum::cs::traits::cs::ConstraintSystem;
use boojum::field::SmallField;
use boojum::gadgets::boolean::Boolean;

use boojum::gadgets::non_native_field::implementations::*;
use boojum::gadgets::num::Num;
use boojum::gadgets::queue::CircuitQueueWitness;
use boojum::gadgets::queue::QueueState;
use boojum::gadgets::traits::allocatable::{CSAllocatableExt, CSPlaceholder};
use boojum::gadgets::traits::round_function::CircuitRoundFunction;
use boojum::gadgets::traits::selectable::Selectable;
use boojum::gadgets::traits::witnessable::WitnessHookable;
use boojum::gadgets::u160::UInt160;
use boojum::gadgets::u256::UInt256;
use boojum::gadgets::u32::UInt32;
use boojum::gadgets::u8::UInt8;
use boojum::pairing::CurveAffine;
use cs_derive::*;
use derivative::Derivative;
use zkevm_opcode_defs::system_params::PRECOMPILE_AUX_BYTE;

use crate::base_structures::log_query::*;
use crate::base_structures::memory_query::*;
use crate::base_structures::precompile_input_outputs::PrecompileFunctionOutputData;
use crate::bn254::ec_add::input::EcAddCircuitInputOutput;
use crate::bn254::ec_pairing::input::EcPairingCircuitInputOutput;
use crate::demux_log_queue::StorageLogQueue;
use crate::ethereum_types::U256;
use crate::fsm_input_output::circuit_inputs::INPUT_OUTPUT_COMMITMENT_LENGTH;
use crate::fsm_input_output::*;
use crate::sha256_round_function::input::Sha256RoundFunctionFSM;
use crate::storage_application::ConditionalWitnessAllocator;

use super::*;

use self::ec_mul::implementation::convert_uint256_to_field_element;
use self::implementation::ec_pairing;
use self::input::EcPairingCircuitInstanceWitness;

pub mod implementation;
pub mod input;

pub const MEMORY_QUERIES_PER_CALL: usize = 6;
pub const NUM_MEMORY_READS_PER_CYCLE: usize = 6;

#[derive(Derivative, CSSelectable)]
#[derivative(Clone, Debug)]
pub struct EcPairingPrecompileCallParams<F: SmallField> {
    pub input_page: UInt32<F>,
    pub input_offset: UInt32<F>,
    pub output_page: UInt32<F>,
    pub output_offset: UInt32<F>,
}

impl<F: SmallField> EcPairingPrecompileCallParams<F> {
    pub fn from_encoding<CS: ConstraintSystem<F>>(_cs: &mut CS, encoding: UInt256<F>) -> Self {
        let input_offset = encoding.inner[0];
        let output_offset = encoding.inner[2];
        let input_page = encoding.inner[4];
        let output_page = encoding.inner[5];

        let new = Self {
            input_page,
            input_offset,
            output_page,
            output_offset,
        };

        new
    }
}

fn ecpairing_precompile_inner<F: SmallField, CS: ConstraintSystem<F>>(
    cs: &mut CS,
    p_x: &UInt256<F>,
    p_y: &UInt256<F>,
    q_x_c0: &UInt256<F>,
    q_x_c1: &UInt256<F>,
    q_y_c0: &UInt256<F>,
    q_y_c1: &UInt256<F>,
) -> (Boolean<F>, Boolean<F>) {
    let base_field_params = &Arc::new(bn254_base_field_params());

    // TODO: add validation of input values
    let p_x = convert_uint256_to_field_element(cs, &p_x, base_field_params);
    let p_y = convert_uint256_to_field_element(cs, &p_y, base_field_params);
    let mut p = BN256SWProjectivePoint::from_xy_unchecked(cs, p_x, p_y);

    let q_x_c0 = convert_uint256_to_field_element(cs, &q_x_c0, base_field_params);
    let q_x_c1 = convert_uint256_to_field_element(cs, &q_x_c1, base_field_params);
    let q_y_c0 = convert_uint256_to_field_element(cs, &q_y_c0, base_field_params);
    let q_y_c1 = convert_uint256_to_field_element(cs, &q_y_c1, base_field_params);

    let q_x = BN256Fq2NNField::new(q_x_c0, q_x_c1);
    let q_y = BN256Fq2NNField::new(q_y_c0, q_y_c1);

    let mut q = BN256SWProjectivePointTwisted::from_xy_unchecked(cs, q_x, q_y);

    let mut result = ec_pairing(cs, &mut p, &mut q);
    let mut one = BN256Fq12NNField::one(cs, base_field_params);
    let paired = result.sub(cs, &mut one).is_zero(cs);

    let success = Boolean::allocated_constant(cs, true);

    (success, paired)
}

pub fn ecpairing_function_entry_point<
    F: SmallField,
    CS: ConstraintSystem<F>,
    R: CircuitRoundFunction<F, 8, 12, 4> + AlgebraicRoundFunction<F, 8, 12, 4>,
>(
    cs: &mut CS,
    witness: EcPairingCircuitInstanceWitness<F>,
    round_function: &R,
    limit: usize,
) -> [Num<F>; INPUT_OUTPUT_COMMITMENT_LENGTH]
where
    [(); <LogQuery<F> as CSAllocatableExt<F>>::INTERNAL_STRUCT_LEN]:,
    [(); <MemoryQuery<F> as CSAllocatableExt<F>>::INTERNAL_STRUCT_LEN]:,
    [(); <UInt256<F> as CSAllocatableExt<F>>::INTERNAL_STRUCT_LEN]:,
    [(); <UInt256<F> as CSAllocatableExt<F>>::INTERNAL_STRUCT_LEN + 1]:,
{
    let EcPairingCircuitInstanceWitness {
        closed_form_input,
        requests_queue_witness,
        memory_reads_witness,
    } = witness;
    let memory_reads_witness: VecDeque<_> = memory_reads_witness.into_iter().flatten().collect();

    let mut structured_input =
        EcPairingCircuitInputOutput::alloc_ignoring_outputs(cs, closed_form_input.clone());
    let start_flag = structured_input.start_flag;

    let requests_queue_state_from_input = structured_input.observable_input.initial_log_queue_state;
    requests_queue_state_from_input.enforce_trivial_head(cs);

    let requests_queue_state_from_fsm = structured_input.hidden_fsm_input.log_queue_state;

    let requests_queue_state = QueueState::conditionally_select(
        cs,
        start_flag,
        &requests_queue_state_from_input,
        &requests_queue_state_from_fsm,
    );

    let mut requests_queue = StorageLogQueue::<F, R>::from_state(cs, requests_queue_state);
    let queue_witness = CircuitQueueWitness::from_inner_witness(requests_queue_witness);
    requests_queue.witness = Arc::new(queue_witness);

    let memory_queue_state_from_input =
        structured_input.observable_input.initial_memory_queue_state;
    memory_queue_state_from_input.enforce_trivial_head(cs);

    let memory_queue_state_from_fsm = structured_input.hidden_fsm_input.memory_queue_state;

    let memory_queue_state = QueueState::conditionally_select(
        cs,
        start_flag,
        &memory_queue_state_from_input,
        &memory_queue_state_from_fsm,
    );

    let mut memory_queue = MemoryQueue::<F, R>::from_state(cs, memory_queue_state);
    let read_queries_allocator = ConditionalWitnessAllocator::<F, UInt256<F>> {
        witness_source: Arc::new(RwLock::new(memory_reads_witness)),
    };

    let precompile_address = UInt160::allocated_constant(
        cs,
        *zkevm_opcode_defs::system_params::ECRECOVER_INNER_FUNCTION_PRECOMPILE_FORMAL_ADDRESS, // TODO: change to ECPAIRING_PRECOMPILE_FORMAL_ADDRESS
    );

    let one_u32 = UInt32::allocated_constant(cs, 1u32);
    let zero_u256 = UInt256::zero(cs);
    let boolean_false = Boolean::allocated_constant(cs, false);
    let boolean_true = Boolean::allocated_constant(cs, true);
    let aux_byte_for_precompile = UInt8::allocated_constant(cs, PRECOMPILE_AUX_BYTE);

    for _cycle in 0..limit {
        let is_empty = requests_queue.is_empty(cs);
        let should_process = is_empty.negated(cs);
        let (request, _) = requests_queue.pop_front(cs, should_process);

        let mut precompile_call_params =
            EcPairingPrecompileCallParams::from_encoding(cs, request.key);

        let timestamp_to_use_for_read = request.timestamp;
        let timestamp_to_use_for_write = timestamp_to_use_for_read.add_no_overflow(cs, one_u32);

        Num::conditionally_enforce_equal(
            cs,
            should_process,
            &Num::from_variable(request.aux_byte.get_variable()),
            &Num::from_variable(aux_byte_for_precompile.get_variable()),
        );
        for (a, b) in request
            .address
            .inner
            .iter()
            .zip(precompile_address.inner.iter())
        {
            Num::conditionally_enforce_equal(
                cs,
                should_process,
                &Num::from_variable(a.get_variable()),
                &Num::from_variable(b.get_variable()),
            );
        }

        let mut read_values = [zero_u256; NUM_MEMORY_READS_PER_CYCLE];
        let mut bias_variable = should_process.get_variable();
        for dst in read_values.iter_mut() {
            let read_query_value = read_queries_allocator.conditionally_allocate_biased(
                cs,
                should_process,
                bias_variable,
            );
            bias_variable = read_query_value.inner[0].get_variable();

            *dst = read_query_value;

            let read_query = MemoryQuery {
                timestamp: timestamp_to_use_for_read,
                memory_page: precompile_call_params.input_page,
                index: precompile_call_params.input_offset,
                rw_flag: boolean_false,
                is_ptr: boolean_false,
                value: read_query_value,
            };

            let _ = memory_queue.push(cs, read_query, should_process);

            precompile_call_params.input_offset = precompile_call_params
                .input_offset
                .add_no_overflow(cs, one_u32);
        }

        let [p_x, p_y, q_x_c0, q_x_c1, q_y_c0, q_y_c1] = read_values;

        if crate::config::CIRCUIT_VERSOBE {
            if should_process.witness_hook(cs)().unwrap() == true {
                dbg!(p_x.witness_hook(cs)());
                dbg!(p_y.witness_hook(cs)());
                dbg!(q_x_c0.witness_hook(cs)());
                dbg!(q_x_c1.witness_hook(cs)());
                dbg!(q_y_c0.witness_hook(cs)());
                dbg!(q_y_c1.witness_hook(cs)());
            }
        }

        let (success, paired) =
            ecpairing_precompile_inner(cs, &p_x, &p_y, &q_x_c0, &q_x_c1, &q_y_c0, &q_y_c1);

        let success_as_u32 = unsafe { UInt32::from_variable_unchecked(success.get_variable()) };
        let mut success = zero_u256;
        success.inner[0] = success_as_u32;

        let paired_as_u32 = unsafe { UInt32::from_variable_unchecked(paired.get_variable()) };
        let mut paired = zero_u256;
        paired.inner[0] = paired_as_u32;

        if crate::config::CIRCUIT_VERSOBE {
            if should_process.witness_hook(cs)().unwrap() == true {
                dbg!(success.witness_hook(cs)());
                dbg!(paired.witness_hook(cs)());
            }
        }

        let success_query = MemoryQuery {
            timestamp: timestamp_to_use_for_write,
            memory_page: precompile_call_params.output_page,
            index: precompile_call_params.output_offset,
            rw_flag: boolean_true,
            is_ptr: boolean_false,
            value: success,
        };

        let _ = memory_queue.push(cs, success_query, should_process);
        precompile_call_params.output_offset = precompile_call_params
            .output_offset
            .add_no_overflow(cs, one_u32);

        let paired_query = MemoryQuery {
            timestamp: timestamp_to_use_for_write,
            memory_page: precompile_call_params.output_page,
            index: precompile_call_params.output_offset,
            rw_flag: boolean_true,
            is_ptr: boolean_false,
            value: paired,
        };

        let _ = memory_queue.push(cs, paired_query, should_process);
    }

    requests_queue.enforce_consistency(cs);

    let done = requests_queue.is_empty(cs);
    structured_input.completion_flag = done;
    structured_input.observable_output = PrecompileFunctionOutputData::placeholder(cs);

    let final_memory_state = memory_queue.into_state();
    let final_requests_state = requests_queue.into_state();

    structured_input.observable_output.final_memory_state = QueueState::conditionally_select(
        cs,
        structured_input.completion_flag,
        &final_memory_state,
        &structured_input.observable_output.final_memory_state,
    );

    structured_input.hidden_fsm_output.log_queue_state = final_requests_state;
    structured_input.hidden_fsm_output.memory_queue_state = final_memory_state;

    structured_input.hook_compare_witness(cs, &closed_form_input);

    let compact_form =
        ClosedFormInputCompactForm::from_full_form(cs, &structured_input, round_function);
    let input_commitment = commit_variable_length_encodable_item(cs, &compact_form, round_function);
    for el in input_commitment.iter() {
        let gate = PublicInputGate::new(el.get_variable());
        gate.add_to_cs(cs);
    }

    input_commitment
}
