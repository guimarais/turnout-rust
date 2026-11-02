//! Immutable, read-only catalogue snapshots and queries.
use crate::{
    layout::{Angle, Diagnostic, Distance, Layout, Position},
    xtp,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

pub const MAX_SOURCES: usize = 64;
pub const MAX_FILES: usize = 1024;
pub const MAX_FILE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct SourceConfig {
    pub paths: Vec<PathBuf>,
    pub cwd: PathBuf,
}
impl SourceConfig {
    pub fn empty() -> Self {
        Self {
            paths: vec![],
            cwd: std::env::current_dir().unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SourceStatus {
    pub configured_path: String,
    pub resolved_path: Option<String>,
    pub kind: Option<String>,
    pub availability: String,
    pub diagnostics: Vec<Diagnostic>,
    pub diagnostics_total: usize,
    pub diagnostics_truncated: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct Location {
    pub path: String,
    pub line: usize,
    pub end_line: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct LocalEndpoint {
    pub endpoint_id: usize,
    pub position: Position,
    pub direction: Angle,
}
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LocalTrackSegment {
    Straight {
        native_index: usize,
        start: Position,
        end: Position,
    },
    Curve {
        native_index: usize,
        center: Position,
        radius: Distance,
        start: Angle,
        sweep: Angle,
    },
}
#[derive(Clone, Debug, Serialize)]
pub struct RouteStep {
    pub native_segment_index: usize,
    pub direction: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct RouteState {
    pub name: String,
    pub traversals: Vec<Vec<RouteStep>>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Part {
    pub part_id: String,
    pub library_id: String,
    pub fingerprint: String,
    pub source_ordinal: usize,
    pub location: Location,
    pub subcontents: Option<String>,
    pub native_type: String,
    pub native_label: String,
    pub native_flags: Option<i64>,
    pub manufacturer: Option<String>,
    pub description: Option<String>,
    pub part_number: Option<String>,
    pub scale: Option<String>,
    pub gauge: Option<Distance>,
    pub category: Option<String>,
    pub category_basis: Option<String>,
    pub support_status: String,
    pub geometry_complete: bool,
    pub routes_complete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<Vec<LocalTrackSegment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Vec<LocalEndpoint>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routes: Option<Vec<RouteState>>,
    pub diagnostics: Vec<Diagnostic>,
    pub diagnostics_total: usize,
    pub diagnostics_truncated: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct Library {
    pub library_id: String,
    pub path: String,
    pub fingerprint: Option<String>,
    pub contents: Option<String>,
    pub aliases: Vec<String>,
    pub parameter_version: Option<u32>,
    pub availability: String,
    pub inventory_complete: bool,
    pub part_count: usize,
    pub complete_part_count: usize,
    pub incomplete_part_count: usize,
    pub selected: bool,
    pub evidence: Vec<serde_json::Value>,
    pub evidence_total: usize,
    pub evidence_truncated: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub diagnostics_total: usize,
    pub diagnostics_truncated: bool,
    #[serde(skip)]
    pub parts: Vec<Part>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Counts {
    pub sources: usize,
    pub libraries: usize,
    pub parts: usize,
    pub matched_tracks: usize,
    pub unmatched_tracks: usize,
    pub ambiguous_tracks: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct CatalogueState {
    pub session_id: String,
    pub snapshot_id: String,
    pub status: String,
    pub selection_origin: Option<String>,
    pub selection_revision: u64,
    pub selected_library_ids: Vec<String>,
    pub inventory_complete: bool,
    pub has_incomplete_definitions: bool,
    pub counts: Counts,
    pub diagnostics: Vec<Diagnostic>,
    pub diagnostics_total: usize,
    pub diagnostics_truncated: bool,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub sources: Vec<SourceStatus>,
    pub libraries: Vec<Library>,
    pub inventory_complete: bool,
    pub diagnostics: Vec<Diagnostic>,
}
#[derive(Clone, Debug)]
pub struct Catalogue {
    pub snapshot: Snapshot,
    pub session_id: String,
    pub selection: BTreeSet<String>,
    pub origin: Option<String>,
    pub revision: u64,
    pub matched: usize,
    pub unmatched: usize,
    pub ambiguous: usize,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn digest(bytes: &[u8]) -> String {
    format!("sha256:{}", hex(&Sha256::digest(bytes)))
}
pub(crate) fn id(kind: &str, fields: &[&[u8]]) -> String {
    let mut h = Sha256::new();
    h.update(b"turnout/catalog/v1\0");
    for f in fields {
        h.update((f.len() as u64).to_be_bytes());
        h.update(f);
    }
    format!("{kind}:sha256:{}", hex(&h.finalize()))
}
fn diag(code: &str, message: impl Into<String>, path: &Path) -> Diagnostic {
    Diagnostic::new(code, message, &path.to_string_lossy(), None).warning()
}

impl Catalogue {
    pub fn open(config: &SourceConfig, layout: &Layout, serial: u64) -> Self {
        let mut paths = Vec::new();
        let mut sources = Vec::new();
        let mut complete = true;
        for configured in &config.paths {
            let p = if configured.is_absolute() {
                configured.clone()
            } else {
                config.cwd.join(configured)
            };
            match fs::metadata(&p) {
                Ok(m) if m.is_file() => paths.push(p),
                Ok(m) if m.is_dir() => match fs::read_dir(&p) {
                    Ok(rd) => {
                        for child in rd.flatten() {
                            let q = child.path();
                            if q.extension().is_some_and(|x| x.eq_ignore_ascii_case("xtp"))
                                && child.file_type().map(|t| t.is_file()).unwrap_or(false)
                            {
                                paths.push(q);
                            }
                        }
                        sources.push(SourceStatus {
                            configured_path: configured.to_string_lossy().into(),
                            resolved_path: Some(p.to_string_lossy().into()),
                            kind: Some("directory".into()),
                            availability: "ready".into(),
                            diagnostics: vec![],
                            diagnostics_total: 0,
                            diagnostics_truncated: false,
                        });
                    }
                    Err(e) => {
                        complete = false;
                        sources.push(SourceStatus {
                            configured_path: configured.to_string_lossy().into(),
                            resolved_path: Some(p.to_string_lossy().into()),
                            kind: Some("directory".into()),
                            availability: "unavailable".into(),
                            diagnostics: vec![diag("source_unavailable", e.to_string(), &p)],
                            diagnostics_total: 1,
                            diagnostics_truncated: false,
                        });
                    }
                },
                _ => {
                    complete = false;
                    sources.push(SourceStatus {
                        configured_path: configured.to_string_lossy().into(),
                        resolved_path: Some(p.to_string_lossy().into()),
                        kind: None,
                        availability: "unavailable".into(),
                        diagnostics: vec![diag(
                            "source_unavailable",
                            "Configured source is missing or not a regular file/directory",
                            &p,
                        )],
                        diagnostics_total: 1,
                        diagnostics_truncated: false,
                    });
                }
            }
        }
        paths.sort();
        paths.dedup();
        if paths.len() > MAX_FILES {
            paths.truncate(MAX_FILES);
            complete = false;
        }
        let mut libraries = Vec::new();
        for p in paths {
            match read_library(&p) {
                Ok(l) => libraries.push(l),
                Err(d) => {
                    complete = false;
                    sources.push(SourceStatus {
                        configured_path: p.to_string_lossy().into(),
                        resolved_path: Some(p.to_string_lossy().into()),
                        kind: Some("file".into()),
                        availability: "unavailable".into(),
                        diagnostics: vec![d],
                        diagnostics_total: 1,
                        diagnostics_truncated: false,
                    });
                }
            }
        }
        let mut c = Self {
            snapshot: Snapshot {
                sources,
                libraries,
                inventory_complete: complete,
                diagnostics: vec![],
            },
            session_id: format!("session-{serial}"),
            selection: BTreeSet::new(),
            origin: None,
            revision: 0,
            matched: 0,
            unmatched: 0,
            ambiguous: 0,
        };
        c.resolve(layout);
        c
    }
    fn resolve(&mut self, layout: &Layout) {
        let mut candidates = BTreeSet::new();
        for track in &layout.tracks {
            let Some(label) = &track.label else {
                self.unmatched += 1;
                continue;
            };
            let Some((m, d, n)) = label
                .split_once('\t')
                .and_then(|(m, x)| x.split_once('\t').map(|(d, n)| (m, d, n)))
            else {
                self.unmatched += 1;
                continue;
            };
            let found: Vec<_> = self
                .snapshot
                .libraries
                .iter()
                .filter(|l| {
                    l.parts.iter().any(|p| {
                        p.scale.as_deref() == track.scale.as_deref()
                            && p.manufacturer.as_deref() == Some(m)
                            && p.description.as_deref() == Some(d)
                            && p.part_number.as_deref() == Some(n)
                    })
                })
                .collect();
            if found.len() == 1 {
                self.matched += 1;
                candidates.insert(found[0].library_id.clone());
            } else if found.is_empty() {
                self.unmatched += 1
            } else {
                self.ambiguous += 1
            }
        }
        if self.snapshot.inventory_complete && self.ambiguous == 0 && !candidates.is_empty() {
            self.selection = candidates;
            self.origin = Some("automatic".into());
        }
    }
    pub fn state(&self) -> CatalogueState {
        let mut selected: Vec<_> = self.selection.iter().cloned().collect();
        selected.sort();
        let libs = self.snapshot.libraries.len();
        let parts = self.snapshot.libraries.iter().map(|x| x.parts.len()).sum();
        let incomplete = self
            .snapshot
            .libraries
            .iter()
            .filter(|l| self.selection.contains(&l.library_id))
            .flat_map(|l| &l.parts)
            .any(|p| p.support_status != "complete");
        let status = if self.origin.as_deref() == Some("explicit") && selected.is_empty() {
            "unselected"
        } else if !selected.is_empty() {
            "selected"
        } else if self.snapshot.libraries.is_empty() && self.snapshot.sources.is_empty() {
            "unconfigured"
        } else if self.ambiguous > 0 || self.matched > 0 {
            "candidates"
        } else {
            "no_match"
        };
        CatalogueState {
            session_id: self.session_id.clone(),
            snapshot_id: digest(
                self.snapshot
                    .libraries
                    .iter()
                    .flat_map(|l| l.library_id.bytes())
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
            status: status.into(),
            selection_origin: self.origin.clone(),
            selection_revision: self.revision,
            selected_library_ids: selected,
            inventory_complete: self.snapshot.inventory_complete,
            has_incomplete_definitions: incomplete,
            counts: Counts {
                sources: self.snapshot.sources.len(),
                libraries: libs,
                parts,
                matched_tracks: self.matched,
                unmatched_tracks: self.unmatched,
                ambiguous_tracks: self.ambiguous,
            },
            diagnostics: self.snapshot.diagnostics.clone(),
            diagnostics_total: self.snapshot.diagnostics.len(),
            diagnostics_truncated: false,
        }
    }
    pub fn select(&mut self, ids: Vec<String>) -> Result<(), Diagnostic> {
        let wanted: BTreeSet<_> = ids.into_iter().collect();
        for x in &wanted {
            let Some(l) = self.snapshot.libraries.iter().find(|l| &l.library_id == x) else {
                return Err(Diagnostic::new(
                    "library_not_found",
                    "Library is not in the current snapshot",
                    "",
                    None,
                ));
            };
            if l.availability == "unavailable" {
                return Err(Diagnostic::new(
                    "library_unavailable",
                    "Library is unavailable",
                    "",
                    None,
                ));
            }
        }
        self.selection = wanted;
        self.origin = Some("explicit".into());
        self.revision += 1;
        Ok(())
    }
    pub fn libraries(&self) -> Vec<Library> {
        self.snapshot
            .libraries
            .iter()
            .cloned()
            .map(|mut l| {
                l.selected = self.selection.contains(&l.library_id);
                l.parts = vec![];
                l
            })
            .collect()
    }
    pub fn parts(&self, filter: &PartFilter) -> Result<Vec<Part>, Diagnostic> {
        if self.selection.is_empty() {
            return Err(Diagnostic::new(
                "catalogue_not_selected",
                "No catalogue library is selected",
                "",
                None,
            ));
        };
        let chosen = filter.library_ids.as_ref().unwrap_or(&self.selection);
        if !chosen.iter().all(|x| self.selection.contains(x)) {
            return Err(Diagnostic::new(
                "library_not_selected",
                "A query library is not selected",
                "",
                None,
            ));
        };
        let mut result = Vec::new();
        for l in &self.snapshot.libraries {
            if chosen.contains(&l.library_id) {
                for p in &l.parts {
                    if filter.matches(p) {
                        let mut q = p.clone();
                        q.geometry = None;
                        q.endpoints = None;
                        q.routes = None;
                        result.push(q)
                    }
                }
            }
        }
        result.sort_by(|a, b| {
            a.library_id
                .cmp(&b.library_id)
                .then(a.source_ordinal.cmp(&b.source_ordinal))
        });
        Ok(result)
    }
    pub fn part(&self, library: &str, part: &str) -> Result<Part, Diagnostic> {
        if !self.selection.contains(library) {
            return Err(Diagnostic::new(
                "library_not_selected",
                "Library is not selected",
                "",
                None,
            ));
        };
        self.snapshot
            .libraries
            .iter()
            .find(|x| x.library_id == library)
            .and_then(|l| l.parts.iter().find(|p| p.part_id == part))
            .cloned()
            .ok_or_else(|| {
                Diagnostic::new(
                    "part_not_found",
                    "Part is absent from the selected library",
                    "",
                    None,
                )
            })
    }
}
#[derive(Clone, Debug, Default)]
pub struct PartFilter {
    pub library_ids: Option<BTreeSet<String>>,
    pub manufacturer: Option<String>,
    pub part_number: Option<String>,
    pub description: Option<String>,
    pub scale: Option<String>,
    pub category: Option<String>,
}
impl PartFilter {
    fn matches(&self, p: &Part) -> bool {
        let sub = |needle: &Option<String>, hay: &Option<String>| {
            needle.as_ref().is_none_or(|n| {
                hay.as_ref()
                    .is_some_and(|h| h.to_lowercase().contains(&n.to_lowercase()))
            })
        };
        sub(&self.manufacturer, &p.manufacturer)
            && sub(&self.part_number, &p.part_number)
            && sub(&self.description, &p.description)
            && self.scale.as_ref().is_none_or(|s| {
                p.scale
                    .as_ref()
                    .is_some_and(|x| x.to_lowercase() == s.to_lowercase())
            })
            && self.category.as_ref().is_none_or(|c| {
                if c == "unknown" {
                    p.category.is_none()
                } else {
                    p.category.as_deref() == Some(c)
                }
            })
    }
}

fn read_library(path: &Path) -> Result<Library, Diagnostic> {
    let bytes = fs::read(path).map_err(|e| diag("source_unavailable", e.to_string(), path))?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err(diag(
            "catalog_limit_exceeded",
            "Parameter file exceeds 8 MiB limit",
            path,
        ));
    }
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let library_id = id("lib", &[canonical.to_string_lossy().as_bytes()]);
    let parsed = xtp::read(path, &bytes, &library_id)?;
    let parts = parsed.parts;
    let complete = parts
        .iter()
        .filter(|p| p.support_status == "complete")
        .count();
    Ok(Library {
        library_id,
        path: canonical.to_string_lossy().into(),
        fingerprint: Some(digest(&bytes)),
        contents: parsed.contents,
        aliases: parsed.aliases,
        parameter_version: parsed.parameter_version,
        availability: if parsed.inventory_complete {
            "ready".into()
        } else {
            "partial".into()
        },
        inventory_complete: parsed.inventory_complete,
        part_count: parts.len(),
        complete_part_count: complete,
        incomplete_part_count: parts.len() - complete,
        selected: false,
        evidence: vec![],
        evidence_total: 0,
        evidence_truncated: false,
        diagnostics: parsed.diagnostics.clone(),
        diagnostics_total: parsed.diagnostics.len(),
        diagnostics_truncated: false,
        parts,
    })
}
