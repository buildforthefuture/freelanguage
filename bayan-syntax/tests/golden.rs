use bayan_syntax::{parse_source, pretty::pretty_module};
use std::fs;
use std::path::PathBuf;

fn examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/examples")
}

#[test]
fn all_examples_parse() {
    let dir = examples_dir();
    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("examples dir")
        .map(|r| r.unwrap().path())
        .filter(|p| p.extension().map(|e| e == "bn").unwrap_or(false))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no example files found");
    for path in files {
        let src = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("failed to read {:?}: {}", path, e)
        });
        let r = parse_source(&src);
        assert!(
            !r.diags.has_errors(),
            "example {:?} failed to parse: {:?}",
            path,
            r.diags.errors
        );
        assert!(r.value.is_some(), "example {:?} produced no AST", path);
    }
}

#[test]
fn fmt_is_idempotent() {
    let dir = examples_dir();
    let files: Vec<_> = fs::read_dir(dir)
        .expect("examples dir")
        .map(|r| r.unwrap().path())
        .filter(|p| p.extension().map(|e| e == "bn").unwrap_or(false))
        .collect();
    for path in files {
        let src = fs::read_to_string(&path).unwrap();
        let r1 = parse_source(&src);
        assert!(r1.value.is_some(), "parse failed for {:?}", path);
        let once = pretty_module(&r1.value.unwrap());
        let r2 = parse_source(&once);
        assert!(
            r2.value.is_some(),
            "re-parse of pretty output failed for {:?}: {:?}",
            path,
            r2.diags.errors
        );
        let twice = pretty_module(&r2.value.unwrap());
        assert_eq!(
            once, twice,
            "pretty-printer not idempotent for {:?}",
            path
        );
    }
}
