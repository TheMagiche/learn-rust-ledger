//! Pretty JSON on disk, written by rename so a crash cannot leave a half file.

use std::fs;
use std::path::{Path, PathBuf};

use spend_core::Ledger;

use crate::{Store, StoreError};

#[derive(Debug, Clone)]
pub struct JsonFileStore {
    path: PathBuf,
}

impl JsonFileStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Store for JsonFileStore {
    fn load(&self) -> Result<Ledger, StoreError> {
        match fs::read_to_string(&self.path) {
            Ok(text) if text.trim().is_empty() => Ok(Ledger::default()),
            Ok(text) => Ok(serde_json::from_str(&text)?),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Ledger::default()),
            Err(err) => Err(StoreError::Io(err)),
        }
    }

    fn save(&self, ledger: &Ledger) -> Result<(), StoreError> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_string_pretty(ledger)?;
        // Write a sibling temp file, then rename over the real path. `rename` on the
        // same directory is atomic, so readers never see a partial ledger.
        let tmp = temp_path(&self.path);
        fs::write(&tmp, json.as_bytes())?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

fn temp_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".tmp");
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use spend_core::{Category, Money, NewExpense};

    use super::*;

    fn dated() -> spend_core::NewExpense {
        NewExpense {
            amount: Money::parse("12.50").expect("amount"),
            category: Category::Food,
            note: Some("lunch".to_string()),
            date: chrono::NaiveDate::from_ymd_opt(2026, 10, 14).expect("date"),
        }
    }

    #[test]
    fn missing_file_is_an_empty_ledger() {
        let dir = tempfile::tempdir().expect("dir");
        let store = JsonFileStore::new(dir.path().join("missing.json"));
        let ledger = store.load().expect("load");
        assert!(ledger.list(&spend_core::Filter::default()).is_empty());
        assert!(!store.path().exists());
    }

    #[test]
    fn round_trip_is_pretty_json() {
        let dir = tempfile::tempdir().expect("dir");
        let path = dir.path().join("nested").join("ledger.json");
        let store = JsonFileStore::new(&path);
        let mut ledger = store.load().expect("load");
        ledger.add(dated()).expect("add");
        ledger.set_budget(Category::Food, Money::parse("150").expect("budget"));
        store.save(&ledger).expect("save");

        let text = fs::read_to_string(&path).expect("read");
        assert!(text.contains('\n'), "pretty JSON is multiple lines");
        assert!(
            !temp_path(&path).exists(),
            "temp file should be renamed away"
        );

        let loaded = JsonFileStore::new(&path).load().expect("reload");
        assert_eq!(loaded.get(1).expect("row").category, Category::Food);
        assert_eq!(
            loaded.budget_status("2026-10")[0].budget,
            Money::parse("150").expect("budget")
        );
    }

    #[test]
    fn corrupt_json_is_an_error() {
        let dir = tempfile::tempdir().expect("dir");
        let path = dir.path().join("ledger.json");
        fs::write(&path, "{").expect("write");
        let error = JsonFileStore::new(path).load().expect_err("corrupt");
        let message = error.to_string();
        assert!(message.contains("invalid ledger json"), "{message}");
    }
}
