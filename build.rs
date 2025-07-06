#![allow(dead_code)]

use std::env;
use std::fs;
use std::path::Path;

#[path = "src/generator.rs"]
mod generator;
#[path = "src/parser.rs"]
mod parser;
#[path = "src/validator.rs"]
mod validator;

fn main() {
    let markdown = env::var("TWIR_MARKDOWN").ok();
    let posts = if let Some(path) = markdown.as_deref() {
        println!("cargo:rerun-if-changed={path}");
        let input = fs::read_to_string(path).expect("failed to read markdown");
        match generator::generate_posts(input) {
            Ok(p) => p,
            Err(e) => panic!("failed to generate posts: {e}"),
        }
    } else {
        println!("cargo:warning=TWIR_MARKDOWN not set, generating empty posts");
        Vec::new()
    };

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest = Path::new(&out_dir).join("generated_posts.rs");
    let mut code = String::from("// Auto-generated file.\n");
    code.push_str("pub const POSTS: &[&str] = &[\n");
    for p in &posts {
        code.push_str("    r#\"");
        code.push_str(p);
        code.push_str("\"#,\n");
    }
    code.push_str("];\n");
    fs::write(&dest, code).expect("failed to write posts file");
    println!("cargo:rerun-if-env-changed=TWIR_MARKDOWN");
}
