use std::fs;

fn workflow(name: &str) -> String {
    fs::read_to_string(format!(".github/workflows/{name}"))
        .unwrap_or_else(|error| panic!("failed to read {name}: {error}"))
}

#[test]
fn production_workflow_preserves_the_marker_across_noop_runs() {
    let production = workflow("prod.yml");

    // The filtered `gh run list` goes through a search index that is
    // intermittently stale, so the previous run must be resolved through the
    // REST endpoint instead.
    assert!(
        !production.contains("gh run list"),
        "production workflow must not resolve the previous run via gh run list (stale index)"
    );
    assert!(
        production
            .contains("repos/${{ github.repository }}/actions/workflows/prod.yml/runs?status=success&branch=main"),
        "production workflow is missing the REST lookup of the previous production run"
    );

    for required in [
        "--repo \"${{ github.repository }}\"",
        "for name in last-sent-prod last-sent; do",
        "last_sent=$(cat last_sent.txt 2>/dev/null || echo \"\")",
        "- name: Preserve last_sent marker when delivery is not needed",
        "if: steps.decide.outputs.should_send == 'false'",
        "- name: Upload preserved last_sent marker",
        "name: last-sent-prod",
        "path: last_sent.txt",
        "if-no-files-found: error",
    ] {
        assert!(
            production.contains(required),
            "production workflow is missing required delivery-state contract: {required}"
        );
    }
}

#[test]
fn delivery_workflow_writes_the_marker_only_after_a_delivery() {
    let delivery = workflow("common-delivery.yml");

    // The filtered `gh run list` goes through a search index that is
    // intermittently stale, so the previous run must be resolved through the
    // REST endpoint instead.
    assert!(
        !delivery.contains("gh run list"),
        "common delivery workflow must not resolve the previous run via gh run list (stale index)"
    );
    assert!(
        delivery
            .contains("repos/${{ github.repository }}/actions/workflows/prod.yml/runs?status=success&branch=main"),
        "common delivery workflow is missing the REST lookup of the previous production run"
    );

    for required in [
        "- name: Save last_sent marker",
        "steps.prepare.outputs.send == 'true' && (inputs.send_main == true || inputs.send_dev == true)",
        "echo \"${{ steps.prepare.outputs.latest_post }}\" > last_sent.txt",
        "- name: Upload last_sent artifact",
        "name: last-sent-${{ env.ARTIFACT_SCOPE }}",
        "path: last_sent.txt",
    ] {
        assert!(
            delivery.contains(required),
            "common delivery workflow is missing required delivery-state contract: {required}"
        );
    }
}

#[test]
fn production_send_is_guarded_by_the_channel_pin_check() {
    let cli = fs::read_to_string("src/cli.rs")
        .unwrap_or_else(|error| panic!("failed to read src/cli.rs: {error}"));

    let guard_position = cli
        .find("pin_guard::enforce_pin_guard")
        .expect("production send must be preceded by the Telegram pin guard");
    let send_position = cli
        .find("Sending posts to production Telegram chat")
        .expect("production send must exist");
    assert!(
        guard_position < send_position,
        "the pin guard must run before the production Telegram send"
    );
}
