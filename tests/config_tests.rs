use turnout::{
    catalog::{Catalogue, SourceConfig},
    xtc,
};

fn layout() -> turnout::layout::Layout {
    xtc::read(std::path::Path::new("tests/fixtures/catalog/one-part.xtc")).unwrap()
}

#[test]
fn directory_sources_are_shallow_and_missing_sources_do_not_break_layout_catalogue() {
    let root = std::env::current_dir().unwrap();
    let config = SourceConfig {
        paths: vec![
            root.join("tests/fixtures/catalog"),
            root.join("tests/fixtures/catalog/missing.xtp"),
        ],
        cwd: root,
    };
    let catalogue = Catalogue::open(&config, &layout(), 1);
    assert!(
        catalogue
            .snapshot
            .libraries
            .iter()
            .any(|library| library.path.ends_with("system-a.xtp"))
    );
    assert!(!catalogue.snapshot.inventory_complete);
    assert!(
        catalogue
            .snapshot
            .sources
            .iter()
            .any(|source| source.availability == "unavailable")
    );
}
