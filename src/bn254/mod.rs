use boojum::gadgets::curves::sw_projective::extended::ExtendedSWProjectivePoint;
use boojum::gadgets::curves::sw_projective::SWProjectivePoint;
use boojum::gadgets::non_native_field::implementations::{
    NonNativeFieldOverU16, NonNativeFieldOverU16Params,
};
use boojum::gadgets::tower_extension::fq6::Fq6;
use boojum::gadgets::tower_extension::params::bn256::{
    BN256Extension12Params, BN256Extension2Params, BN256Extension6Params,
};
use boojum::gadgets::tower_extension::{fq12::Fq12, fq2::Fq2};

// Characteristic of the base field for bn256 curve
pub use boojum::pairing::bn256::fq::Fq as BN256Fq;
// Order of group of points for bn256 curve
pub use boojum::pairing::bn256::fr::Fr as BN256Fr;

// Affine point for bn256 curve
pub use boojum::pairing::bn256::G1Affine as BN256Affine;
pub use boojum::pairing::bn256::G2Affine as BN256AffineTwisted;

pub mod fixed_base_mul_table;

pub mod ec_add;
pub mod ec_mul;
pub mod ec_pairing;
pub mod tests;
