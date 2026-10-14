//! The binary and the HTTP API share one JSON file.

use std::process::Command;

fn spend(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_spend"))
        .args(args)
        .output()
        .expect("run spend")
}

#[test]
fn add_list_total_report_remove_and_errors() {
    let dir = tempfile::tempdir().expect("dir");
    let file = dir.path().join("ledger.json");
    let file_arg = file.to_str().expect("utf8 path");

    let added = spend(&[
        "--file", file_arg, "add", "12.50", "food", "--note", "lunch",
    ]);
    assert!(
        added.status.success(),
        "{}",
        String::from_utf8_lossy(&added.stderr)
    );
    let stdout = String::from_utf8_lossy(&added.stdout);
    assert!(stdout.contains("added"), "{stdout}");
    assert!(stdout.contains("12.50"), "{stdout}");
    assert!(stdout.contains("lunch"), "{stdout}");

    let listed = spend(&["--file", file_arg, "list", "--search", "lunch"]);
    assert!(listed.status.success());
    assert!(String::from_utf8_lossy(&listed.stdout).contains("food"));

    let total = spend(&["--file", file_arg, "total"]);
    assert_eq!(String::from_utf8_lossy(&total.stdout).trim(), "12.50");

    let budget = spend(&["--file", file_arg, "budget", "set", "food", "10.00"]);
    assert!(
        budget.status.success(),
        "{}",
        String::from_utf8_lossy(&budget.stderr)
    );
    assert!(String::from_utf8_lossy(&budget.stdout).contains("over"));

    let report = spend(&["--file", file_arg, "report"]);
    assert!(report.status.success());
    let report_text = String::from_utf8_lossy(&report.stdout);
    assert!(report_text.contains("food"), "{report_text}");
    assert!(report_text.contains("budgets"), "{report_text}");

    let removed = spend(&["--file", file_arg, "remove", "1"]);
    assert!(
        removed.status.success(),
        "{}",
        String::from_utf8_lossy(&removed.stderr)
    );
    assert!(String::from_utf8_lossy(&removed.stdout).contains("removed"));

    let missing = spend(&["--file", file_arg, "remove", "1"]);
    assert_eq!(missing.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&missing.stderr);
    assert!(stderr.starts_with("error: "), "{stderr}");
    assert!(stderr.contains("expense 1 not found"), "{stderr}");

    let zero = spend(&["--file", file_arg, "add", "0", "food"]);
    assert_eq!(zero.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&zero.stderr).contains("amount must be greater than zero"));

    let bad = spend(&["--file", file_arg, "add", "abc", "food"]);
    assert!(String::from_utf8_lossy(&bad.stderr).contains("invalid amount: abc"));
}
