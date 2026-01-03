use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::PathBuf;
use pyo3::{pyfunction, pymodule, wrap_pyfunction, Bound, PyResult};
use pyo3::prelude::PyModuleMethods;
use pyo3::types::PyModule;
use rayon::prelude::*;
use regex::Regex;
use walkdir::WalkDir;

fn build_module_map(root: &PathBuf) -> HashMap<String, PathBuf> {
    let module_re = Regex::new(
        r"(?m)^\s*module\s*([a-zA-Z_][a-zA-Z0-9_]*)"
    ).unwrap();

    WalkDir::new(root)
        .into_iter()
        .par_bridge()
        .filter_map(|e| e.ok())
        .filter(|e| {
            matches!(
                e.path().extension()
                .and_then(|e| e.to_str()),
                Some("v") | Some("sv")
            )
        })
        .filter_map(|entry| {
            let content = fs::read_to_string(entry.path()).unwrap_or_default();
            let mut local = Vec::new();

            for cap in module_re.captures_iter(&content) {
                let name = cap[1].to_string();
                local.push((name, entry.path().to_path_buf()));
            }
            Some(local)
        })
        .flatten()
        .collect::<Vec<(String, PathBuf)>>()
        .into_iter()
        .fold(
            HashMap::new(),
            |mut acc, (name, path)| {
                acc.entry(name).or_insert(path);
                acc
            }
        )
}

fn extract_instantiated_modules(content: &str) -> Vec<String> {
        let inst_re =
        Regex::new(r"(?m)^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*(?:#\s*\([^;]*\))?\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\(")
            .unwrap();

    inst_re
        .captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}

fn resolve_filelist(top: &str, module_map: &HashMap<String, PathBuf>) -> Vec<PathBuf> {
    let mut visited_modules = HashSet::new();
    let mut visited_files = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back(top.to_string());

    while let Some(module) = queue.pop_front() {
        if !visited_modules.insert(module.clone()) {
            continue;
        }

        let Some(path) = module_map.get(&module) else {
            continue; // blackbox / 未找到
        };

        if !visited_files.insert(path.clone()) {
            continue;
        }

        let content = fs::read_to_string(path).unwrap_or_default();
        let sub_modules = extract_instantiated_modules(&content);

        for sub in sub_modules {
            if module_map.contains_key(&sub) {
                queue.push_back(sub);
            }
        }
    }

    let mut result: Vec<_> = visited_files.into_iter().collect();
    result.sort();
    result
}

#[pyfunction]
fn get(root: String, top_module: String) -> PyResult<Vec<String>> {
    let module_map = build_module_map(&PathBuf::from(&root));
    let file_list = resolve_filelist(&top_module, &module_map)
        .iter().map(|f| f.to_string_lossy().to_string())
        .collect();
    Ok(file_list)
}

#[pymodule]
fn mex(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get, m)?)?;
    Ok(())
}
