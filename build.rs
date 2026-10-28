// Embeds `plugins/specs/*.toml` into the binary as a compile-time
// `(slug, toml_src)` table (OUT_DIR/specs.rs). The deployed binary runs
// from `work/<site>/`, never the repo root — runtime path resolution is
// impossible, so specs are baked in and parsed at REGISTRY init.
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let dir = Path::new("plugins/specs");
    println!("cargo:rerun-if-changed=plugins/specs");
    let mut specs: Vec<(String, String)> = Vec::new();
    if dir.is_dir() {
        let mut names: Vec<_> = fs::read_dir(dir)
            .expect("read plugins/specs")
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
    let mut out = String::from("pub(crate) static SPEC_SRCS: &[(&str, &str)] = &[\n");
    for (slug, src) in &specs {
        out.push_str(&format!(
            "    ({:?}, {:?}),\n",
            slug, src
        ));
    }
    out.push_str("];\n");
    fs::write(Path::new(&env::var("OUT_DIR").unwrap()).join("specs.rs"), out)
        .expect("write specs.rs");
}
