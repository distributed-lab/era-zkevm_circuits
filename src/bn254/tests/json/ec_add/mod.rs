use std::{fs::File, io::Read};

use serde::{Deserialize, Serialize};

use crate::bn254::tests::json::types::RawG1Point;

/// Path to the test cases for EC addition
const EC_ADD_TEST_CASES_PATH: &str = "./src/bn254/tests/json/ec_add/ecadd_tests.json";

// --- EC Add tests ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ECAddTestCase {
    pub point_1: RawG1Point,
    pub point_2: RawG1Point,
    pub expected: RawG1Point,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ECAddTestCases {
    pub tests: Vec<ECAddTestCase>,
}

/// Load EC addition test cases from the file
pub(in super::super) fn load_ec_add_test_cases() -> ECAddTestCases {
    let mut file = File::open(EC_ADD_TEST_CASES_PATH).expect("Unable to open the file");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Unable to parse to string");
    let test_cases: ECAddTestCases = serde_json::from_str(&data).expect("Failed to deserialize");

    test_cases
}

const TEST_DATA: &str = include_str!("ecadd_tests.json");


