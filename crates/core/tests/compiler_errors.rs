//! Each teaching snippet must produce the error code we show in the UI.

use std::process::Command;

#[test]
fn snippets_match_rustc() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../web/src/learn/compilerErrors.json"
    );
    let text = std::fs::read_to_string(path).expect("compilerErrors.json");
    let cases: serde_json::Value = serde_json::from_str(&text).expect("json");
    let cases = cases.as_array().expect("array");
    assert!(cases.len() >= 8, "need at least 8 compiler errors");

    for case in cases {
        let id = case["id"].as_str().expect("id");
        let code = case["code"].as_str().expect("code");
        let broken = case["broken"].as_str().expect("broken");
        let fix = case["fix"].as_str().expect("fix");

        let broken_out = compile_snippet(broken);
        let stderr = String::from_utf8_lossy(&broken_out.stderr);
        if code == "unused_must_use" {
            assert!(
                broken_out.status.success(),
                "{id} should warn, not error:\n{stderr}"
            );
            assert!(
                stderr.contains("unused `Result` that must be used"),
                "{id} stderr:\n{stderr}"
            );
        } else {
            assert!(
                !broken_out.status.success(),
                "{id} should fail to compile:\n{stderr}"
            );
            let needle = format!("error[{code}]");
            assert!(stderr.contains(&needle), "{id} stderr:\n{stderr}");
        }

        let fixed = compile_snippet(fix);
        let fix_stderr = String::from_utf8_lossy(&fixed.stderr);
        assert!(
            fixed.status.success(),
            "{id} fix should compile:\n{fix_stderr}"
        );
    }
}

fn compile_snippet(source: &str) -> std::process::Output {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("example.rs");
    std::fs::write(&file, source).expect("write snippet");
    Command::new("rustc")
        .args(["--edition", "2021", "--error-format", "short"])
        .arg("-o")
        .arg(dir.path().join("out"))
        .arg(&file)
        .output()
        .expect("rustc")
}
