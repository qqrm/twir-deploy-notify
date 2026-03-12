#[path = "common.rs"]
mod common;
#[path = "support/golden.rs"]
mod golden_support;

#[test]
fn fixture_2025_06_25_matches_golden() {
    golden_support::assert_fixture_matches_golden("2025-06-25-this-week-in-rust.md");
}

#[test]
fn fixture_2025_07_02_matches_golden() {
    golden_support::assert_fixture_matches_golden("2025-07-02-this-week-in-rust.md");
}

#[test]
fn fixture_2025_10_22_matches_golden() {
    golden_support::assert_fixture_matches_golden("2025-10-22-this-week-in-rust.md");
}

#[test]
fn fixture_call_for_participation_matches_golden() {
    golden_support::assert_fixture_matches_golden("2025-07-05-call-for-participation.md");
}
