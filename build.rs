use std::{env, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=stdlib");

    let out_dir = env::var("OUT_DIR").unwrap();
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

    fs::create_dir_all(Path::new(&out_dir).join("stdlib")).unwrap();
    fs::write(Path::new(&out_dir).join("stdlib.rs"), generated).unwrap();
}
