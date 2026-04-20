use std::{
    fs,
    path::{Path, PathBuf},
};

use twir_deploy_notify::generator::generate_posts;

use crate::common::assert_valid_markdown;

pub fn assert_fixture_matches_golden(fixture_name: &str) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = root.join("tests").join(fixture_name);
    let snapshot_dir = root
        .join("tests")
        .join("golden")
        .join(snapshot_name(fixture_name));

    let input = fs::read_to_string(&fixture_path)
        .map(normalize_line_endings)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {e}", fixture_path.display()));
    let actual = generate_posts(input)
        .unwrap_or_else(|e| panic!("failed to generate posts for {fixture_name}: {e}"));
    let expected = read_snapshot_posts(&snapshot_dir);

    assert_eq!(
        actual.len(),
        expected.len(),
        "snapshot post count mismatch for {fixture_name}"
    );

    for (index, (actual_post, expected_post)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            actual_post,
            expected_post,
            "snapshot mismatch for {fixture_name} output_{}.md",
            index + 1
        );
        assert_valid_markdown(actual_post);
    }
}

fn snapshot_name(fixture_name: &str) -> &str {
    fixture_name.strip_suffix(".md").unwrap_or(fixture_name)
}

fn read_snapshot_posts(dir: &Path) -> Vec<String> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read snapshot dir {}: {e}", dir.display()))
        .map(|entry| entry.expect("read_dir entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();

    paths.sort_by(|left, right| {
        left.file_name()
            .expect("snapshot file name")
            .cmp(right.file_name().expect("snapshot file name"))
    });

    assert!(
        !paths.is_empty(),
        "expected at least one snapshot file in {}",
        dir.display()
    );

    paths
        .into_iter()
        .map(|path| {
            fs::read_to_string(&path)
                .map(normalize_line_endings)
                .unwrap_or_else(|e| panic!("failed to read snapshot {}: {e}", path.display()))
        })
        .collect()
}

fn normalize_line_endings(input: String) -> String {
    input.replace("\r\n", "\n")
}
