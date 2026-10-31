// Embeds `plugins/specs/*.toml` into the binary as a compile-time
// `(slug, toml_src)` table (OUT_DIR/specs.rs). The deployed binary runs
// from `work/<site>/`, never the repo root — runtime path resolution is
// impossible, so specs are baked in and parsed at REGISTRY init.
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    emit_specs("plugins/specs", "SPEC_SRCS", "specs.rs");
    emit_specs("themes/specs", "THEME_SPEC_SRCS", "theme_specs.rs");
}

/// One `<dir>/*.toml` → `(slug, src)` const table in OUT_DIR.
fn emit_specs(dir_name: &str, sym: &str, out_name: &str) {
    let dir = Path::new(dir_name);
    println!("cargo:rerun-if-changed={dir_name}");
    let mut specs: Vec<(String, String)> = Vec::new();
    if dir.is_dir() {
        let mut names: Vec<_> = fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("read {dir_name}: {e}"))
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|e| e == "toml"))
            .collect();
        names.sort();
        for p in names {
            let slug = p.file_stem().unwrap().to_string_lossy().into_owned();
            let src = fs::read_to_string(&p)
                .unwrap_or_else(|e| panic!("read {}: {e}", p.display()));
            specs.push((slug, src));
        }
    }
    let mut out = format!("pub(crate) static {sym}: &[(&str, &str)] = &[\n");
    for (slug, src) in &specs {
        out.push_str(&format!("    ({:?}, {:?}),\n", slug, src));
    }
    out.push_str("];\n");
    fs::write(Path::new(&env::var("OUT_DIR").unwrap()).join(out_name), out)
        .unwrap_or_else(|e| panic!("write {out_name}: {e}"));
}
