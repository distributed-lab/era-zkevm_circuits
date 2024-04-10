use self::input::EcMulCircuitInstanceWitness;

use super::*;
use crate::base_structures::log_query::*;
use crate::base_structures::memory_query::*;
use crate::base_structures::precompile_input_outputs::PrecompileFunctionOutputData;
use crate::bn254::ec_mul::input::EcMulCircuitInputOutput;
use crate::demux_log_queue::StorageLogQueue;
use crate::ethereum_types::U256;
use crate::fsm_input_output::circuit_inputs::INPUT_OUTPUT_COMMITMENT_LENGTH;
use crate::fsm_input_output::*;
use arrayvec::ArrayVec;
use boojum::algebraic_props::round_function::AlgebraicRoundFunction;
use boojum::crypto_bigint::{Zero, U1024};
use boojum::cs::gates::ConstantAllocatableCS;
use boojum::cs::traits::cs::ConstraintSystem;
use boojum::field::SmallField;
use boojum::gadgets::boolean::Boolean;
use boojum::gadgets::curves::sw_projective::SWProjectivePoint;
use boojum::gadgets::keccak256::keccak256;
use boojum::gadgets::non_native_field::implementations::*;
use boojum::gadgets::num::Num;
use boojum::gadgets::queue::CircuitQueueWitness;
use boojum::gadgets::queue::QueueState;
use boojum::gadgets::traits::allocatable::{CSAllocatableExt, CSPlaceholder};
use boojum::gadgets::traits::round_function::CircuitRoundFunction;
use boojum::gadgets::traits::selectable::Selectable;
use boojum::gadgets::traits::witnessable::WitnessHookable;
use boojum::gadgets::u16::UInt16;
use boojum::gadgets::u160::UInt160;
use boojum::gadgets::u256::UInt256;
use boojum::gadgets::u32::UInt32;
use boojum::gadgets::u8::UInt8;
use cs_derive::*;
use derivative::Derivative;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use zkevm_opcode_defs::system_params::PRECOMPILE_AUX_BYTE;

pub mod input;
mod test;

// Characteristic of the base field for bn256 curve
use boojum::pairing::bn256::fq::Fq as BN256Fq;
// Order of group of points for bn256 curve
use boojum::pairing::bn256::fr::Fr as BN256Fr;

pub const MEMORY_QUERIES_PER_CALL: usize = 4;

#[derive(Derivative, CSSelectable)]
#[derivative(Clone, Debug)]
pub struct EcMulPrecompileCallParams<F: SmallField> {
    pub input_page: UInt32<F>,
    pub input_offset: UInt32<F>,
    pub output_page: UInt32<F>,
    pub output_offset: UInt32<F>,
}

impl<F: SmallField> EcMulPrecompileCallParams<F> {
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

fn bn254_base_field_params() -> BN256BaseNNFieldParams {
    NonNativeFieldOverU16Params::create()
}

fn bn254_scalar_field_params() -> BN256ScalarNNFieldParams {
    NonNativeFieldOverU16Params::create()
}

// pub fn ecmul_function_entry_point<
//     F: SmallField,
//     CS: ConstraintSystem<F>,
//     R: CircuitRoundFunction<F, 8, 12, 4> + AlgebraicRoundFunction<F, 8, 12, 4>,
// >(
//     cs: &mut CS,
//     witness: EcMulCircuitInstanceWitness<F>,
//     limit: usize,
// ) -> [Num<F>; INPUT_OUTPUT_COMMITMENT_LENGTH]
// where
//     [(); <LogQuery<F> as CSAllocatableExt<F>>::INTERNAL_STRUCT_LEN]:,
//     [(); <MemoryQuery<F> as CSAllocatableExt<F>>::INTERNAL_STRUCT_LEN]:,
//     [(); <UInt256<F> as CSAllocatableExt<F>>::INTERNAL_STRUCT_LEN]:,
//     [(); <UInt256<F> as CSAllocatableExt<F>>::INTERNAL_STRUCT_LEN + 1]:,
// {
//     assert!(limit <= u32::MAX as usize);

//     let EcMulCircuitInstanceWitness {
//         closed_form_input,
//         requests_queue_witness,
//         memory_reads_witness,
//     } = witness;

//     let memory_reads_witness: VecDeque<_> = memory_reads_witness.into_iter().flatten().collect();

//     let precompile_address = UInt160::allocated_constant(
//         cs,
//         *zkevm_opcode_defs::system_params::ECRECOVER_INNER_FUNCTION_PRECOMPILE_FORMAL_ADDRESS,
//     );
//     let aux_byte_for_precompile = UInt8::allocated_constant(cs, PRECOMPILE_AUX_BYTE);

//     let scalar_params = Arc::new(bn254_scalar_field_params());
//     let base_params = Arc::new(bn254_base_field_params());

//     let mut structured_input =
//         EcMulCircuitInputOutput::alloc_ignoring_outputs(cs, closed_form_input.clone());
//     let start_flag = structured_input.start_flag;

//     let requests_queue_state_from_input = structured_input.observable_input.initial_log_queue_state;

//     // it must be trivial
//     requests_queue_state_from_input.enforce_trivial_head(cs);

//     let requests_queue_state_from_fsm = structured_input.hidden_fsm_input.log_queue_state;

//     let requests_queue_state = QueueState::conditionally_select(
//         cs,
//         start_flag,
//         &requests_queue_state_from_input,
//         &requests_queue_state_from_fsm,
//     );

//     let memory_queue_state_from_input =
//         structured_input.observable_input.initial_memory_queue_state;

//     // it must be trivial
//     memory_queue_state_from_input.enforce_trivial_head(cs);

//     let memory_queue_state_from_fsm = structured_input.hidden_fsm_input.memory_queue_state;

//     let memory_queue_state = QueueState::conditionally_select(
//         cs,
//         start_flag,
//         &memory_queue_state_from_input,
//         &memory_queue_state_from_fsm,
//     );

//     let mut requests_queue = StorageLogQueue::<F, R>::from_state(cs, requests_queue_state);
//     let queue_witness = CircuitQueueWitness::from_inner_witness(requests_queue_witness);
//     requests_queue.witness = Arc::new(queue_witness);

//     let mut memory_queue = MemoryQueue::<F, R>::from_state(cs, memory_queue_state);

//     let one_u32 = UInt32::allocated_constant(cs, 1u32);
//     let zero_u256 = UInt256::zero(cs);
//     let boolean_false = Boolean::allocated_constant(cs, false);
//     let boolean_true = Boolean::allocated_constant(cs, true);

//     use crate::storage_application::ConditionalWitnessAllocator;
//     let read_queries_allocator = ConditionalWitnessAllocator::<F, UInt256<F>> {
//         witness_source: Arc::new(RwLock::new(memory_reads_witness)),
//     };

//     for _cycle in 0..limit {
//         let is_empty = requests_queue.is_empty(cs);
//         let should_process = is_empty.negated(cs);
//         let (request, _) = requests_queue.pop_front(cs, should_process);

//         let mut precompile_call_params =
//             EcMulPrecompileCallParams::from_encoding(cs, request.key);

//         let timestamp_to_use_for_read = request.timestamp;
//         let timestamp_to_use_for_write = timestamp_to_use_for_read.add_no_overflow(cs, one_u32);

//         Num::conditionally_enforce_equal(
//             cs,
//             should_process,
//             &Num::from_variable(request.aux_byte.get_variable()),
//             &Num::from_variable(aux_byte_for_precompile.get_variable()),
//         );
//         for (a, b) in request
//             .address
//             .inner
//             .iter()
//             .zip(precompile_address.inner.iter())
//         {
//             Num::conditionally_enforce_equal(
//                 cs,
//                 should_process,
//                 &Num::from_variable(a.get_variable()),
//                 &Num::from_variable(b.get_variable()),
//             );
//         }

//         let mut read_values = [zero_u256; NUM_MEMORY_READS_PER_CYCLE];
//         let mut bias_variable = should_process.get_variable();
//         for dst in read_values.iter_mut() {
//             let read_query_value: UInt256<F> = read_queries_allocator
//                 .conditionally_allocate_biased(cs, should_process, bias_variable);
//             bias_variable = read_query_value.inner[0].get_variable();

//             *dst = read_query_value;

//             let read_query = MemoryQuery {
//                 timestamp: timestamp_to_use_for_read,
//                 memory_page: precompile_call_params.input_page,
//                 index: precompile_call_params.input_offset,
//                 rw_flag: boolean_false,
//                 is_ptr: boolean_false,
//                 value: read_query_value,
//             };

//             let _ = memory_queue.push(cs, read_query, should_process);

//             precompile_call_params.input_offset = precompile_call_params
//                 .input_offset
//                 .add_no_overflow(cs, one_u32);
//         }

//         let [message_hash_as_u256, v_as_u256, r_as_u256, s_as_u256] = read_values;
//         let rec_id = v_as_u256.inner[0].to_le_bytes(cs)[0];

//         if crate::config::CIRCUIT_VERSOBE {
//             if should_process.witness_hook(cs)().unwrap() == true {
//                 dbg!(rec_id.witness_hook(cs)());
//                 dbg!(r_as_u256.witness_hook(cs)());
//                 dbg!(s_as_u256.witness_hook(cs)());
//                 dbg!(message_hash_as_u256.witness_hook(cs)());
//             }
//         }

//         let (success, written_value) = ecrecover_precompile_inner_routine::<_, _, ALLOW_ZERO_MESSAGE>(
//             cs,
//             &rec_id,
//             &r_as_u256,
//             &s_as_u256,
//             &message_hash_as_u256,
//             valid_x_in_external_field.clone(),
//             valid_y_in_external_field.clone(),
//             valid_t_in_external_field.clone(),
//             &base_params,
//             &scalar_params,
//         );

//         let success_as_u32 = unsafe { UInt32::from_variable_unchecked(success.get_variable()) };
//         let mut success_as_u256 = zero_u256;
//         success_as_u256.inner[0] = success_as_u32;

//         if crate::config::CIRCUIT_VERSOBE {
//             if should_process.witness_hook(cs)().unwrap() == true {
//                 dbg!(success_as_u256.witness_hook(cs)());
//                 dbg!(written_value.witness_hook(cs)());
//             }
//         }

//         let success_query = MemoryQuery {
//             timestamp: timestamp_to_use_for_write,
//             memory_page: precompile_call_params.output_page,
//             index: precompile_call_params.output_offset,
//             rw_flag: boolean_true,
//             value: success_as_u256,
//             is_ptr: boolean_false,
//         };

//         precompile_call_params.output_offset = precompile_call_params
//             .output_offset
//             .add_no_overflow(cs, one_u32);

//         let _ = memory_queue.push(cs, success_query, should_process);

//         let value_query = MemoryQuery {
//             timestamp: timestamp_to_use_for_write,
//             memory_page: precompile_call_params.output_page,
//             index: precompile_call_params.output_offset,
//             rw_flag: boolean_true,
//             value: written_value,
//             is_ptr: boolean_false,
//         };

//         let _ = memory_queue.push(cs, value_query, should_process);
//     }

//     requests_queue.enforce_consistency(cs);

//     // form the final state
//     let done = requests_queue.is_empty(cs);
//     structured_input.completion_flag = done;
//     structured_input.observable_output = PrecompileFunctionOutputData::placeholder(cs);

//     let final_memory_state = memory_queue.into_state();
//     let final_requets_state = requests_queue.into_state();

//     structured_input.observable_output.final_memory_state = QueueState::conditionally_select(
//         cs,
//         structured_input.completion_flag,
//         &final_memory_state,
//         &structured_input.observable_output.final_memory_state,
//     );

//     structured_input.hidden_fsm_output.log_queue_state = final_requets_state;
//     structured_input.hidden_fsm_output.memory_queue_state = final_memory_state;

//     // self-check
//     structured_input.hook_compare_witness(cs, &closed_form_input);

//     use boojum::cs::gates::PublicInputGate;

//     let compact_form =
//         ClosedFormInputCompactForm::from_full_form(cs, &structured_input, round_function);
//     let input_commitment = commit_variable_length_encodable_item(cs, &compact_form, round_function);
//     for el in input_commitment.iter() {
//         let gate = PublicInputGate::new(el.get_variable());
//         gate.add_to_cs(cs);
//     }

//     input_commitment
// }
