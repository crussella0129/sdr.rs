//! Documentation check: the README reference catalog lists the mesh references.

#[test]
fn test_readme_lists_aredn() {
    let readme = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../README.md"));
    assert!(
        readme.contains("aredn/aredn"),
        "README reference catalog is missing the AREDN entry"
    );
    assert!(
        readme.contains("rfc8966") || readme.contains("RFC 8966") || readme.contains("Babel"),
        "README reference catalog is missing the Babel (RFC 8966) entry"
    );
}
