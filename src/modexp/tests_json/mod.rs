use lazy_static::lazy_static;

pub mod u2048;
pub mod u256;

// All tests gathered in one place
lazy_static! {
    /// Test cases for modexp
    pub static ref MODEXP_32_BYTES_TEST_CASES: u256::Modexp32BytesTestCases = u256::load_modexp_32_bytes_test_cases();
    /// Test cases for modmul
    pub static ref MODMUL_32_BYTES_TEST_CASES: u256::Modmul32BytesTestCases = u256::load_modmul_32_bytes_test_cases();
    /// Test cases for modmul
    pub static ref MODMUL_256_BYTES_TEST_CASES: u2048::Modmul256BytesTestCases = u2048::load_modmul_256_bytes_test_cases();
}
