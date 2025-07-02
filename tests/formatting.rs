use std::fs;
use std::process::Command;

#[test]
fn crate_of_the_week_formatting() {
    let input = "Title: TWIR 605\nNumber: 605\nDate: 2025-06-25\n\n## Crate of the Week\n\nThis week's crate is [primitive_fixed_point_decimal](https://docs.rs/primitive_fixed_point_decimal), a crate of *real* fixed-point decimal types.\n";
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("input.md");
    fs::write(&input_path, input).unwrap();

    let exe = env!("CARGO_BIN_EXE_twir-deploy-notify");
    let status = Command::new(exe)
        .current_dir(&dir)
        .arg(&input_path)
        .status()
        .expect("failed to run binary");
    assert!(status.success());

    let output_path = dir.path().join("output_1.md");
    let contents = fs::read_to_string(output_path).unwrap();
    let expected = "*Часть 1/1*\n**TWIR 605** — \\#605 — 2025\\-06\\-25\n\n\\-\\-\\-\n\n📰 **CRATE OF THE WEEK**\nThis week's crate is [primitive\\_fixed\\_point\\_decimal](https://docs.rs/primitive_fixed_point_decimal), a crate of real fixed\\-point decimal types\\.\n\n\\-\\-\\-\n\nПолный выпуск: [https://this\\-week\\-in\\-rust\\.org/blog/2025/06/25/this\\-week\\-in\\-rust\\-605/](https://this-week-in-rust.org/blog/2025/06/25/this-week-in-rust-605/)";
    assert_eq!(contents, expected);
}
