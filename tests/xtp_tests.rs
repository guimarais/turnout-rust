use std::fs;
use turnout::xtp;

#[test]
fn parses_authored_four_part_library_with_local_geometry_and_routes() {
    let path = std::path::Path::new("tests/fixtures/catalog/system-a.xtp");
    let parsed = xtp::read(path, &fs::read(path).unwrap(), "lib:sha256:test").unwrap();
    assert_eq!(parsed.contents.as_deref(), Some("System A"));
    assert_eq!(parsed.parts.len(), 4);
    let straight = &parsed.parts[0];
    assert_eq!(straight.manufacturer.as_deref(), Some("Märklin"));
    assert_eq!(straight.part_number.as_deref(), Some("100"));
    assert_eq!(straight.category.as_deref(), Some("straight"));
    assert!(straight.geometry_complete);
    assert!(straight.routes_complete);
    assert_eq!(straight.geometry.as_ref().unwrap().len(), 1);
    assert_eq!(parsed.parts[3].category.as_deref(), Some("crossing"));
    assert_eq!(
        parsed.parts[3].routes.as_ref().unwrap()[0].traversals.len(),
        2
    );
}

#[test]
fn rejects_invalid_parameter_encoding_and_version() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad.xtp");
    fs::write(&bad, [0xff]).unwrap();
    assert_eq!(
        xtp::read(&bad, &fs::read(&bad).unwrap(), "id")
            .unwrap_err()
            .code,
        "invalid_parameter_utf8"
    );
    let version = dir.path().join("version.xtp");
    fs::write(&version, "CONTENTS x\nPARAM 13\n").unwrap();
    assert_eq!(
        xtp::read(&version, &fs::read(&version).unwrap(), "id")
            .unwrap_err()
            .code,
        "unsupported_parameter_version"
    );
}
