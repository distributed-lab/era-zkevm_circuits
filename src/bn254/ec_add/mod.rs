use boojum::gadgets::{curves::bn256::{BN256BaseNNFieldParams, BN256ScalarNNFieldParams}, non_native_field::implementations::NonNativeFieldOverU16Params};

pub mod test;

fn bn254_base_field_params() -> BN256BaseNNFieldParams {
    NonNativeFieldOverU16Params::create()
}

fn bn254_scalar_field_params() -> BN256ScalarNNFieldParams {
    NonNativeFieldOverU16Params::create()
}