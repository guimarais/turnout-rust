use std::fs;
use turnout::{
    catalog::{Catalogue, PartFilter, SourceConfig},
    xtc,
};

fn catalogue() -> Catalogue {
    let root = std::env::current_dir().unwrap();
    let config = SourceConfig {
        paths: vec![root.join("tests/fixtures/catalog/system-a.xtp")],
        cwd: root,
    };
    let layout = xtc::read(std::path::Path::new("tests/fixtures/catalog/one-part.xtc")).unwrap();
    Catalogue::open(&config, &layout, 1)
}

#[test]
fn unique_match_selects_library_and_exposes_unplaced_definitions() {
    let catalogue = catalogue();
    let state = catalogue.state();
    assert_eq!(state.status, "selected");
    assert_eq!(state.selected_library_ids.len(), 1);
    let parts = catalogue.parts(&PartFilter::default()).unwrap();
    assert_eq!(parts.len(), 4);
    assert!(
        parts
            .iter()
            .any(|p| p.part_number.as_deref() == Some("103"))
    );
}

#[test]
fn explicit_selection_is_atomic_and_clear_is_explicit() {
    let mut catalogue = catalogue();
    let original = catalogue.state().selected_library_ids;
    assert!(catalogue.select(vec!["lib:sha256:missing".into()]).is_err());
    assert_eq!(catalogue.state().selected_library_ids, original);
    catalogue.select(vec![]).unwrap();
    assert_eq!(catalogue.state().status, "unselected");
    assert!(catalogue.parts(&PartFilter::default()).is_err());
}

#[test]
fn filters_keep_duplicate_source_specific_parts_distinct() {
    let catalogue = catalogue();
    let filter = PartFilter {
        manufacturer: Some("mär".into()),
        ..Default::default()
    };
    assert_eq!(catalogue.parts(&filter).unwrap().len(), 4);
    assert_eq!(
        catalogue
            .parts(&PartFilter {
                category: Some("curve".into()),
                ..filter
            })
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn opening_does_not_modify_parameter_source() {
    let path = std::path::Path::new("tests/fixtures/catalog/system-a.xtp");
    let before = fs::read(path).unwrap();
    let mtime = fs::metadata(path).unwrap().modified().unwrap();
    let _ = catalogue();
    assert_eq!(fs::read(path).unwrap(), before);
    assert_eq!(fs::metadata(path).unwrap().modified().unwrap(), mtime);
}
