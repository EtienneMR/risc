use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn bundle_stdlib(out_dir: &Path) {
    println!("cargo:rerun-if-changed=stdlib");

    let mut generated = String::new();

    generated.push_str("pub fn get(name: &str) -> Option<&'static str> {\n");
    generated.push_str("    match name {\n");

    for entry in fs::read_dir("stdlib").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        let stem = path.file_stem().unwrap().to_string_lossy();
        let content = fs::read_to_string(&path).unwrap();

        generated.push_str(&format!("        {stem:?} => Some({content:?}),\n"));
    }

    generated.push_str("        _ => None,\n");
    generated.push_str("    }\n");
    generated.push_str("}\n");

    fs::write(out_dir.join("stdlib.rs"), generated).unwrap();
}

fn generate_tests(out_dir: &Path) {
    println!("cargo:rerun-if-changed=tests");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let tests_dir = manifest_dir.join("tests");

    let mut tests = String::new();

    let mut entries: Vec<_> = fs::read_dir(&tests_dir)
        .unwrap()
        .map(Result::unwrap)
        .collect();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("ri") {
            continue;
        }
        if !path.is_file() {
            continue;
        }

        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap();
        let fn_name = format!("test_{stem}");

        tests.push_str(&format!(
            r#"
#[test]
fn {fn_name}() -> Result<(), ()> {{
    test_ri("tests/{}")
}}
"#,
            path.file_name().unwrap().to_string_lossy()
        ));
    }

    fs::write(out_dir.join("ri_tests.rs"), tests).unwrap();
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::create_dir_all(&out_dir).unwrap();

    bundle_stdlib(&out_dir);
    generate_tests(&out_dir);
}
