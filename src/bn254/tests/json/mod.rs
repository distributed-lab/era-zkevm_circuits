use std::{fs::File, io::Read};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use super::utils::{RawFq2, RawFq6, RawPoint};

lazy_static! {
    /// Test cases for EC addition
    pub static ref EC_ADD_TEST_CASES: ECAddTestCases = load_ec_add_test_cases();
    /// Test cases for scalar decomposition
    pub static ref DECOMPOSITION_TEST_CASES: DecompositionTestCases = load_decomposition_test_cases();
    /// Test cases for scalar multiplication
    pub static ref EC_MUL_TEST_CASES: MultiplicationTestCases = load_multiplication_test_cases();
    /// Test cases for `Fq2` operations
    pub static ref FQ2_TEST_CASES: Fq2TestCases = load_fq2_test_cases();
    /// Test cases for `Fq6` operations
    pub static ref FQ6_TEST_CASES: Fq6TestCases = load_fq6_test_cases();
}

/// Path to the test cases for EC addition
const EC_ADD_TEST_CASES_PATH: &str = "./src/bn254/tests/json/ecadd_tests.json";
/// Path to the test cases for scalar decomposition
const DECOMPOSITION_TEST_CASES_PATH: &str = "./src/bn254/tests/json/decomposition_tests.json";
/// Path to the test cases for scalar multiplication
const EC_MUL_TEST_CASES_PATH: &str = "./src/bn254/tests/json/ecmul_tests.json";
/// Path to the test cases for Fq2 operations
const FQ2_TEST_CASES_PATH: &str = "./src/bn254/tests/json/fq2_tests.json";
/// Path to the test cases for Fq6 operations
const FQ6_TEST_CASES_PATH: &str = "./src/bn254/tests/json/fq6_tests.json";

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
    let test_cases: ECAddTestCases = serde_json::from_str(&data).expect("Failed to deserialize");

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

// --- Fq2 tests ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Fq2TestCase {
    pub scalar_1: RawFq2,
    pub scalar_2: RawFq2,
    pub expected: Fq2ExpectedValue,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Fq2ExpectedValue {
    pub sum: RawFq2,
    pub difference: RawFq2,
    pub product: RawFq2,
    pub quotient: RawFq2,
    pub scalar_1_non_residue: RawFq2,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Fq2TestCases {
    pub tests: Vec<Fq2TestCase>,
}

/// Load Fq2 test cases from the file
fn load_fq2_test_cases() -> Fq2TestCases {
    let mut file = File::open(FQ2_TEST_CASES_PATH).expect("Unable to open the file");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Unable to parse to string");
    let test_cases: Fq2TestCases = serde_json::from_str(&data).expect("Failed to deserialize");

    test_cases
}

// --- Fq6 Test Cases ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Fq6TestCase {
    pub scalar_1: RawFq6,
    pub scalar_2: RawFq6,
    pub c0: RawFq2,
    pub c1: RawFq2,
    pub expected: Fq6ExpectedValue,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Fq6ExpectedValue {
    pub sum: RawFq6,
    pub difference: RawFq6,
    pub product: RawFq6,
    pub quotient: RawFq6,
    pub product_c1: RawFq6,
    pub product_c0c1: RawFq6,
    pub scalar_1_inverse: RawFq6,
    pub scalar_1_square: RawFq6,
    pub scalar_1_non_residue: RawFq6,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Fq6TestCases {
    pub tests: Vec<Fq6TestCase>,
}

/// Load `Fq6` test cases from the file
fn load_fq6_test_cases() -> Fq6TestCases {
    let mut file = File::open(FQ6_TEST_CASES_PATH).expect("Unable to open the file");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Unable to parse to string");
    let test_cases: Fq6TestCases = serde_json::from_str(&data).expect("Failed to deserialize");

    test_cases
}
