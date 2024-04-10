use std::{fs::File, io::Read};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use super::utils::RawPoint;

lazy_static! {
    /// Test cases for EC addition
    pub static ref EC_ADD_TEST_CASES: ECAddTestCases = load_ec_add_test_cases();
    /// Test cases for scalar decomposition
    pub static ref DECOMPOSITION_TEST_CASES: DecompositionTestCases = load_decomposition_test_cases();
    /// Test cases for scalar multiplication
    pub static ref EC_MUL_TEST_CASES: MultiplicationTestCases = load_multiplication_test_cases();
}

/// Path to the test cases for EC addition
const EC_ADD_TEST_CASES_PATH: &str = "./src/bn254/tests/json/ecadd_tests.json";
/// Path to the test cases for scalar decomposition
const DECOMPOSITION_TEST_CASES_PATH: &str = "./src/bn254/tests/json/decomposition_tests.json";
/// Path to the test cases for scalar multiplication
const EC_MUL_TEST_CASES_PATH: &str = "./src/bn254/tests/json/ecmul_tests.json";

// --- EC add tests ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ECAddTestCase {
    pub point_1: RawPoint,
    pub point_2: RawPoint,
    pub expected: RawPoint,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ECAddTestCases {
    pub tests: Vec<ECAddTestCase>,
}

/// Load EC addition test cases from the file
fn load_ec_add_test_cases() -> ECAddTestCases {
    let mut file = File::open(EC_ADD_TEST_CASES_PATH).expect("Unable to open the file");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Unable to parse to string");
    let test_cases: ECAddTestCases =
        serde_json::from_str(&data).expect("Failed to deserialize");

    test_cases
}

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
fn load_decomposition_test_cases() -> DecompositionTestCases {
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
    pub point: RawPoint,
    pub scalar: String,
    pub expected: RawPoint,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MultiplicationTestCases {
    pub tests: Vec<MultiplicationTestCase>,
}

/// Load scalar multiplication test cases from the file
fn load_multiplication_test_cases() -> MultiplicationTestCases {
    let mut file = File::open(EC_MUL_TEST_CASES_PATH).expect("Unable to open the file");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Unable to parse to string");
    let test_cases: MultiplicationTestCases =
        serde_json::from_str(&data).expect("Failed to deserialize");

    test_cases
}