//! Pull handler functions and `Ledger` methods into `web/src/learn`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use syn::{ImplItem, Item, Type, Visibility};

fn main() -> ExitCode {
    match generate() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(1)
        }
    }
}

fn generate() -> Result<(), String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|err| err.to_string())?;
    let handlers = root.join("crates/api/src/handlers.rs");
    let ledger = root.join("crates/core/src/ledger.rs");
    let handler_src = fs::read_to_string(&handlers).map_err(|err| err.to_string())?;
    let ledger_src = fs::read_to_string(&ledger).map_err(|err| err.to_string())?;

    let mut sources: BTreeMap<String, Source> = BTreeMap::new();
    for name in public_functions(&handler_src)? {
        let (line, code) = extract_function(&handler_src, &name)
            .ok_or_else(|| format!("could not extract handler {name}"))?;
        sources.insert(
            name,
            Source {
                file: "crates/api/src/handlers.rs".to_string(),
                start_line: line,
                code,
            },
        );
    }
    for name in ledger_methods(&ledger_src)? {
        let (line, code) = extract_function(&ledger_src, &name)
            .ok_or_else(|| format!("could not extract Ledger::{name}"))?;
        sources.insert(
            format!("Ledger::{name}"),
            Source {
                file: "crates/core/src/ledger.rs".to_string(),
                start_line: line,
                code,
            },
        );
    }

    let used_in = learn_sites(&root.join("crates"))?;
    let learn_dir = root.join("web/src/learn");
    fs::create_dir_all(&learn_dir).map_err(|err| err.to_string())?;
    fs::write(learn_dir.join("rustSources.ts"), render_sources(&sources))
        .map_err(|err| err.to_string())?;
    fs::write(learn_dir.join("usedIn.ts"), render_used_in(&used_in))
        .map_err(|err| err.to_string())?;
    println!(
        "wrote {} rust sources and {} concept sites",
        sources.len(),
        used_in.values().map(Vec::len).sum::<usize>()
    );
    Ok(())
}

struct Source {
    file: String,
    start_line: usize,
    code: String,
}

fn public_functions(src: &str) -> Result<Vec<String>, String> {
    let file = syn::parse_file(src).map_err(|err| err.to_string())?;
    let mut names = Vec::new();
    for item in file.items {
        if let Item::Fn(func) = item {
            if matches!(func.vis, Visibility::Public(_)) {
                names.push(func.sig.ident.to_string());
            }
        }
    }
    Ok(names)
}

fn ledger_methods(src: &str) -> Result<Vec<String>, String> {
    let file = syn::parse_file(src).map_err(|err| err.to_string())?;
    let mut names = Vec::new();
    for item in file.items {
        let Item::Impl(block) = item else {
            continue;
        };
        if block.trait_.is_some() || !type_is_ledger(&block.self_ty) {
            continue;
        }
        for item in block.items {
            if let ImplItem::Fn(method) = item {
                names.push(method.sig.ident.to_string());
            }
        }
    }
    Ok(names)
}

fn type_is_ledger(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    path.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "Ledger")
}

fn learn_sites(crates: &Path) -> Result<BTreeMap<String, Vec<String>>, String> {
    let mut sites: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut files = Vec::new();
    collect_rs(crates, &mut files)?;
    for path in files {
        let text = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        let relative = path
            .strip_prefix(crates.parent().unwrap_or(crates))
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        for (line_number, line) in text.lines().enumerate() {
            let mut rest = line;
            while let Some(start) = rest.find("LEARN(") {
                let after = &rest[start + "LEARN(".len()..];
                let Some(end) = after.find(')') else {
                    break;
                };
                let id = &after[..end];
                if !id.is_empty()
                    && id
                        .chars()
                        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
                {
                    sites
                        .entry(id.to_string())
                        .or_default()
                        .push(format!("{relative}:{}", line_number + 1));
                }
                rest = &after[end + 1..];
            }
        }
    }
    for locations in sites.values_mut() {
        locations.sort();
        locations.dedup();
    }
    Ok(sites)
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|err| err.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            collect_rs(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

fn extract_function(src: &str, name: &str) -> Option<(usize, String)> {
    let needle = format!("fn {name}");
    let mut search_from = 0;
    while let Some(rel) = src[search_from..].find(&needle) {
        let start = search_from + rel;
        let after = start + needle.len();
        if !boundary_before(src, start) || !boundary_after(src, after) {
            search_from = after;
            continue;
        }
        let brace = find_body_brace(src, after)?;
        let end = matching_brace(src, brace)?;
        let code_start = include_leading(src, start);
        let line = src[..code_start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1;
        return Some((line, src[code_start..end].to_string()));
    }
    None
}

fn boundary_before(src: &str, index: usize) -> bool {
    match src[..index].chars().next_back() {
        None => true,
        Some(ch) => !ch.is_ascii_alphanumeric() && ch != '_',
    }
}

fn boundary_after(src: &str, index: usize) -> bool {
    match src[index..].chars().next() {
        None => true,
        Some(ch) => !ch.is_ascii_alphanumeric() && ch != '_',
    }
}

fn include_leading(src: &str, fn_index: usize) -> usize {
    let prefix = &src[..fn_index];
    let mut line_start = prefix.rfind('\n').map(|index| index + 1).unwrap_or(0);
    let mut keep = line_start;
    loop {
        if line_start == 0 {
            break;
        }
        let prev_end = line_start - 1;
        let prev_start = src[..prev_end]
            .rfind('\n')
            .map(|index| index + 1)
            .unwrap_or(0);
        let line = src[prev_start..prev_end].trim();
        if line.is_empty() {
            if prev_start == 0 {
                break;
            }
            line_start = prev_start;
            continue;
        }
        if line.starts_with("//") || line.starts_with("///") || line.starts_with("#[") {
            keep = prev_start;
            if prev_start == 0 {
                break;
            }
            line_start = prev_start;
            continue;
        }
        break;
    }
    keep
}

fn find_body_brace(src: &str, from: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut index = from;
    while index < bytes.len() {
        match bytes[index] {
            b'{' => return Some(index),
            b'/' if index + 1 < bytes.len() && bytes[index + 1] == b'/' => {
                index = skip_line(bytes, index);
            }
            _ => index += 1,
        }
    }
    None
}

fn matching_brace(src: &str, open: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut depth = 0_i32;
    let mut index = open;
    while index < bytes.len() {
        match bytes[index] {
            b'/' if index + 1 < bytes.len() && bytes[index + 1] == b'/' => {
                index = skip_line(bytes, index);
            }
            b'/' if index + 1 < bytes.len() && bytes[index + 1] == b'*' => {
                index = skip_block_comment(bytes, index)?;
            }
            b'"' => index = skip_string(bytes, index)?,
            b'\'' => index = skip_char(bytes, index)?,
            b'{' => {
                depth += 1;
                index += 1;
            }
            b'}' => {
                depth -= 1;
                index += 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => index += 1,
        }
    }
    None
}

fn skip_line(bytes: &[u8], index: usize) -> usize {
    bytes
        .iter()
        .skip(index)
        .position(|byte| *byte == b'\n')
        .map(|offset| index + offset + 1)
        .unwrap_or(bytes.len())
}

fn skip_block_comment(bytes: &[u8], index: usize) -> Option<usize> {
    let rest = bytes.get(index + 2..)?;
    rest.windows(2)
        .position(|pair| pair == b"*/")
        .map(|offset| index + 2 + offset + 2)
}

fn skip_string(bytes: &[u8], index: usize) -> Option<usize> {
    let mut cursor = index + 1;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => cursor += 2,
            b'"' => return Some(cursor + 1),
            _ => cursor += 1,
        }
    }
    None
}

fn skip_char(bytes: &[u8], index: usize) -> Option<usize> {
    let mut cursor = index + 1;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => cursor += 2,
            b'\'' => return Some(cursor + 1),
            _ => cursor += 1,
        }
    }
    None
}

fn render_sources(sources: &BTreeMap<String, Source>) -> String {
    let mut value = serde_json::Map::new();
    for (name, source) in sources {
        let mut object = serde_json::Map::new();
        object.insert(
            "file".to_string(),
            serde_json::Value::String(source.file.clone()),
        );
        object.insert(
            "startLine".to_string(),
            serde_json::Value::Number(source.start_line.into()),
        );
        object.insert(
            "code".to_string(),
            serde_json::Value::String(source.code.clone()),
        );
        value.insert(name.clone(), serde_json::Value::Object(object));
    }
    format!(
        "{}\n\nexport type RustSource = {{\n  file: string\n  startLine: number\n  code: string\n}}\n\nexport const rustSources: Record<string, RustSource> = {}\n",
        generated_banner(),
        serde_json::to_string_pretty(&serde_json::Value::Object(value)).unwrap_or_else(|_| "{{}}".to_string())
    )
}

fn render_used_in(sites: &BTreeMap<String, Vec<String>>) -> String {
    format!(
        "{}\n\nexport const usedInByConcept: Record<string, string[]> = {}\n",
        generated_banner(),
        serde_json::to_string_pretty(sites).unwrap_or_else(|_| "{{}}".to_string())
    )
}

fn generated_banner() -> &'static str {
    "/* eslint-disable */\n// Generated by `make gen` (tools/gen-sources). Do not edit by hand."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_a_function_with_its_comment() {
        let src = "fn other() {}\n\n// LEARN(option): none means missing\npub fn get(&self) -> Option<()> {\n    None\n}\n";
        let (line, code) = extract_function(src, "get").expect("function");
        assert_eq!(line, 3);
        assert!(code.contains("LEARN(option)"));
        assert!(code.ends_with('}'));
        assert!(!code.contains("fn other"));
    }
}
