use std::{fs::File, io::Read};

use serde::{Deserialize, Serialize};

use crate::bn254::tests::types::RawG1Point;

/// Path to the test cases for scalar decomposition
const DECOMPOSITION_TEST_CASES_PATH: &str =
    "./src/bn254/tests/json/ec_mul/decomposition_tests.json";
/// Path to the test cases for scalar multiplication
const EC_MUL_TEST_CASES_PATH: &str = "./src/bn254/tests/json/ec_mul/ecmul_tests.json";

// --- Scalar decomposition tests ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DecompositionTestCase {
    pub k: String,
    pub k1: String,
    pub k2: String,
    pub k1_negated: bool,
    pub k2_negated: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DecompositionTestCases {
    pub tests: Vec<DecompositionTestCase>,
}

/// Load scalar decomposition test cases from the file
pub(in super::super) fn load_decomposition_test_cases() -> DecompositionTestCases {
    let mut file = File::open(DECOMPOSITION_TEST_CASES_PATH).expect("Unable to open the file");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Unable to parse to string");
    let test_cases: DecompositionTestCases =
        serde_json::from_str(&data).expect("Failed to deserialize");

    test_cases
}

// --- EC multiplication tests ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MultiplicationTestCase {
    pub point: RawG1Point,
    pub scalar: String,
    pub expected: RawG1Point,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MultiplicationTestCases {
    pub tests: Vec<MultiplicationTestCase>,
}

/// Load scalar multiplication test cases from the file
pub(in super::super) fn load_multiplication_test_cases() -> MultiplicationTestCases {
    let mut file = File::open(EC_MUL_TEST_CASES_PATH).expect("Unable to open the file");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Unable to parse to string");
    let test_cases: MultiplicationTestCases =
        serde_json::from_str(&data).expect("Failed to deserialize");

    test_cases
}
