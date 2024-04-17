use std::{fs::File, io::Read};

use serde::{Deserialize, Serialize};

use crate::bn254::tests::types::{RawFq12, RawG1Point, RawG2Point};

/// Path to the test cases for G2 Curve
const G2_CURVE_TEST_CASES_PATH: &str = "./src/bn254/tests/json/ec_pairing/g2_tests.json";
/// Path to the test cases for line/tangent functions evaluation
const LINE_FUNCTION_TEST_CASES_PATH: &str =
    "./src/bn254/tests/json/ec_pairing/line_functions_tests.json";
/// Path to the test cases for easy exponentiation
const FINAL_EXP_TEST_CASES_PATH: &str = "./src/bn254/tests/json/ec_pairing/final_exp_tests.json";

/// --- G2 Tests ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct G2TestCase {
    pub point_1: RawG2Point,
    pub point_2: RawG2Point,
    pub expected: G2ExpectedValue,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct G2ExpectedValue {
    pub sum: RawG2Point,
    pub point_1_double: RawG2Point,
    pub point_2_double: RawG2Point,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct G2TestCases {
    pub tests: Vec<G2TestCase>,
}

/// Load `G2Curve` test cases from the file
pub(in super::super) fn load_g2_curve_test_cases() -> G2TestCases {
    let mut file = File::open(G2_CURVE_TEST_CASES_PATH).expect("Unable to open the file");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Unable to parse to string");
    let test_cases: G2TestCases = serde_json::from_str(&data).expect("Failed to deserialize");

    test_cases
}

// --- Line function tests ---
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LineFunctionTestCase {
    pub g2_point_1: RawG2Point,
    pub g2_point_2: RawG2Point,
    pub g1_point: RawG1Point,
    pub expected: LineFunctionExpectedValue,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LineFunctionExpectedValue {
    pub line_add: RawFq12,
    pub line_tangent_1: RawFq12,
    pub line_tangent_2: RawFq12,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LineFunctionTestCases {
    pub tests: Vec<LineFunctionTestCase>,
}

/// Load `G2Curve` test cases from the file
pub(in super::super) fn load_line_function_test_cases() -> LineFunctionTestCases {
    let mut file = File::open(LINE_FUNCTION_TEST_CASES_PATH).expect("Unable to open the file");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Unable to parse to string");
    let test_cases: LineFunctionTestCases =
        serde_json::from_str(&data).expect("Failed to deserialize");

    test_cases
}

// --- Final exponentiation tests ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalExpTestCase {
    pub scalar: RawFq12,
    pub expected: RawFq12,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalExpTestCases {
    pub tests: Vec<FinalExpTestCase>,
}

/// Load `EasyExp` test cases from the file
pub(in super::super) fn load_final_exp_test_cases() -> FinalExpTestCases {
    let mut file = File::open(FINAL_EXP_TEST_CASES_PATH).expect("Unable to open the file");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Unable to parse to string");
    let test_cases: FinalExpTestCases = serde_json::from_str(&data).expect("Failed to deserialize");

    test_cases
}
