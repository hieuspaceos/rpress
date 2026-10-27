//! Every shipped template must parse; a syntax slip in base_fr/base_es
//! would only surface at render time otherwise.
use minijinja::Environment;

#[test]
fn templates_parse() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
    let mut env = Environment::new();
    for entry in walkdir_files(&root) {
        if entry.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        let name = entry.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
        if name.starts_with("reference/") || name.starts_with("admin/") {
            continue; // captured origin HTML / admin assets, not templates
        }
        let src = std::fs::read_to_string(&entry).unwrap();
        env.add_template_owned(name.clone(), src)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
    }
}

fn walkdir_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            out.extend(walkdir_files(&p));
        } else {
            out.push(p);
        }
    }
    out
}
