use std::fs;

fn workflow(name: &str) -> String {
    fs::read_to_string(format!(".github/workflows/{name}"))
        .unwrap_or_else(|error| panic!("failed to read {name}: {error}"))
}

#[test]
fn production_workflow_preserves_the_marker_across_noop_runs() {
    let production = workflow("prod.yml");

    for required in [
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
