//! Make sure `web/dist` exists so `rust-embed` can compile before `make build`.

use std::fs;
use std::path::Path;

fn main() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dist = manifest.join("../../web/dist");
    if !dist.join("index.html").exists() {
        fs::create_dir_all(&dist).expect("create web/dist");
        fs::write(
            dist.join("index.html"),
            "<!doctype html><title>spend</title><p>UI not built. Run make dev or make build.</p>\n",
        )
        .expect("write placeholder index");
    }
    println!("cargo:rerun-if-changed={}", dist.display());
    if let Ok(entries) = fs::read_dir(&dist) {
        for entry in entries.flatten() {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
    }
}
