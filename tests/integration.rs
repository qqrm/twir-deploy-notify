use std::fs;
use std::process::Command;

#[test]
fn crate_of_the_week_is_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let input = "Title: This Week in Rust 605\nNumber: 605\nDate: 2025-06-25\n\n## Crate of the Week\nThis week's crate is [primitive_fixed_point_decimal](https://docs.rs/primitive_fixed_point_decimal), a crate of *real* fixed-point decimal types.\n\nThanks to [Wu Bingzheng](https://users.rust-lang.org/t/crate-of-the-week/2704/1445) for the self-suggestion!\n\n[Please submit your suggestions and votes for next week][submit_crate]!\n\n[submit_crate]: https://users.rust-lang.org/t/crate-of-the-week/2704\n";
    let input_path = dir.path().join("input.md");
    fs::write(&input_path, input).unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_twir-deploy-notify"))
        .arg(&input_path)
        .current_dir(dir.path())
        .status()
        .expect("failed to run binary");
    assert!(status.success());

    let output = fs::read_to_string(dir.path().join("output_1.md")).unwrap();
    assert!(output.contains("📰 **CRATE OF THE WEEK**"));
    assert!(output.contains("primitive\\_fixed\\_point\\_decimal"));
}

#[test]
fn crate_of_week_followed_by_section() {
    let dir = tempfile::tempdir().unwrap();
    let input = "Title: Test\nNumber: 1\nDate: 2024-01-01\n\n## Crate of the Week\nThis week's crate is [demo](https://example.com).\n\n## Next\n- item\n";
    let input_path = dir.path().join("input.md");
    fs::write(&input_path, input).unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_twir-deploy-notify"))
        .arg(&input_path)
        .current_dir(dir.path())
        .status()
        .expect("failed to run binary");
    assert!(status.success());

    let combined = fs::read_to_string(dir.path().join("output_1.md")).unwrap();
    assert!(combined.contains("📰 **CRATE OF THE WEEK**"));
    assert!(combined.contains("[demo](https://example.com)"));
    assert!(combined.contains("📰 **NEXT**"));
}
