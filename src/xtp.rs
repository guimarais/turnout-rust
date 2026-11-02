//! Bounded reader for the supported XTrackCAD parameter-file subset.
use crate::{
    catalog::{
        LocalEndpoint, LocalTrackSegment, Location, Part, RouteState, RouteStep, digest, id,
    },
    layout::{Angle, Diagnostic, Distance, Position},
};
use std::path::Path;
#[derive(Debug)]
pub struct Parsed {
    pub contents: Option<String>,
    pub aliases: Vec<String>,
    pub parameter_version: Option<u32>,
    pub parts: Vec<Part>,
    pub inventory_complete: bool,
    pub diagnostics: Vec<Diagnostic>,
}
fn d(code: &str, msg: impl Into<String>, path: &Path, line: usize) -> Diagnostic {
    Diagnostic::new(code, msg, &path.to_string_lossy(), Some(line)).warning()
}
pub fn read(path: &Path, bytes: &[u8], library_id: &str) -> Result<Parsed, Diagnostic> {
    let text = std::str::from_utf8(bytes).map_err(|e| {
        Diagnostic::new(
            "invalid_parameter_utf8",
            e.to_string(),
            &path.to_string_lossy(),
            None,
        )
    })?;
    let lines: Vec<_> = text.lines().enumerate().map(|(n, x)| (n + 1, x)).collect();
    let mut contents = None;
    let mut aliases = vec![];
    let mut version = None;
    let mut parts = vec![];
    let mut diagnostics = vec![];
    let mut group = None;
    let mut i = 0;
    let mut ordinal = 0;
    while i < lines.len() {
        let (line, raw) = lines[i];
        let t = raw.trim();
        i += 1;
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if let Some(v) = t.strip_prefix("CONTENTS ") {
            let label = v.trim_matches('"').to_owned();
            if contents.is_none() {
                contents = Some(label)
            } else {
                aliases.push(label)
            }
            continue;
        }
        if let Some(v) = t.strip_prefix("PARAM ") {
            if version.is_some() {
                return Err(Diagnostic::new(
                    "malformed_parameter_file",
                    "Duplicate PARAM directive",
                    &path.to_string_lossy(),
                    Some(line),
                ));
            }
            let n = v.parse::<u32>().map_err(|_| {
                Diagnostic::new(
                    "malformed_parameter_file",
                    "Invalid PARAM directive",
                    &path.to_string_lossy(),
                    Some(line),
                )
            })?;
            if n != 12 {
                return Err(Diagnostic::new(
                    "unsupported_parameter_version",
                    format!("Unsupported PARAM {n}"),
                    &path.to_string_lossy(),
                    Some(line),
                ));
            }
            version = Some(n);
            continue;
        }
        if let Some(v) = t.strip_prefix("SUBCONTENTS ") {
            group = Some(v.trim_matches('"').to_owned());
            continue;
        }
        if !t.starts_with("TURNOUT ") {
            if t.starts_with("INCLUDE") {
                diagnostics.push(d(
                    "unsupported_parameter_record",
                    "INCLUDE is not followed",
                    path,
                    line,
                ));
            }
            continue;
        }
        ordinal += 1;
        let header = t;
        let mut p = make_part(path, library_id, ordinal, line, header, group.clone());
        let mut segments = vec![];
        let mut endpoints = vec![];
        let mut routes = vec![];
        let mut complete = true;
        let mut ended = false;
        while i < lines.len() {
            let (n, r) = lines[i];
            let x = r.trim();
            i += 1;
            if x == "END" || x == "END$SEGS" {
                ended = true;
                break;
            }
            let f: Vec<_> = x.split_whitespace().collect();
            match f.first().copied() {
                Some("E") => {
                    if let Some(e) = endpoint(&f, endpoints.len() + 1) {
                        endpoints.push(e)
                    } else {
                        complete = false;
                        p.diagnostics
                            .push(d("malformed_definition", "Invalid endpoint", path, n));
                    }
                }
                Some("S") => {
                    if let Some(s) = straight(&f, segments.len() + 1) {
                        segments.push(s)
                    } else {
                        complete = false;
                        p.diagnostics
                            .push(d("malformed_definition", "Invalid straight", path, n));
                    }
                }
                Some("C") => {
                    if let Some(c) = curve(&f, segments.len() + 1) {
                        segments.push(c)
                    } else {
                        complete = false;
                        p.diagnostics
                            .push(d("malformed_definition", "Invalid curve", path, n));
                    }
                }
                Some("P") => match route(&f) {
                    Some(r) => routes.push(r),
                    None => {
                        complete = false;
                        p.diagnostics
                            .push(d("invalid_route", "Invalid route", path, n));
                    }
                },
                Some("U") | Some("L") | Some("A") | Some("G") | Some("F") | Some("Y")
                | Some("L3") | Some("A3") | Some("G3") | Some("F3") | Some("F4") | Some("Y3")
                | Some("Y4") => {}
                Some("TURNOUT") => {
                    complete = false;
                    p.diagnostics.push(d(
                        "malformed_definition",
                        "Definition ended before terminator",
                        path,
                        n,
                    ));
                    i -= 1;
                    break;
                }
                _ => {
                    complete = false;
                    p.diagnostics.push(d(
                        "unsupported_definition",
                        "Unsupported definition record",
                        path,
                        n,
                    ));
                }
            }
        }
        if !ended {
            complete = false;
            p.diagnostics.push(d(
                "malformed_definition",
                "Missing definition terminator",
                path,
                line,
            ));
        }
        if endpoints.len() < 2 || segments.is_empty() {
            complete = false
        }
        p.geometry_complete = complete && endpoints.len() >= 2 && !segments.is_empty();
        p.routes_complete = complete && !routes.is_empty();
        if !p.routes_complete {
            p.diagnostics.push(d(
                "routes_unavailable",
                "Routes are unavailable or incomplete",
                path,
                line,
            ));
        }
        p.support_status = if p.geometry_complete && p.routes_complete {
            "complete".into()
        } else if !segments.is_empty() || !endpoints.is_empty() {
            "partial".into()
        } else {
            "unsupported".into()
        };
        p.category = category(
            &segments,
            &endpoints,
            &routes,
            p.geometry_complete && p.routes_complete,
        );
        p.category_basis = p.category.as_ref().map(|_| "geometry_and_routes".into());
        p.location.end_line = lines.get(i.saturating_sub(1)).map(|x| x.0).unwrap_or(line);
        p.geometry = p.geometry_complete.then_some(segments);
        p.endpoints = p.geometry_complete.then_some(endpoints);
        p.routes = p.routes_complete.then_some(routes);
        p.diagnostics_total = p.diagnostics.len();
        parts.push(p);
    }
    if contents.is_none() {
        return Err(Diagnostic::new(
            "malformed_parameter_file",
            "First non-comment record must be CONTENTS",
            &path.to_string_lossy(),
            None,
        ));
    }
    Ok(Parsed {
        contents,
        aliases,
        parameter_version: version,
        parts,
        inventory_complete: true,
        diagnostics,
    })
}
fn finite(x: Option<&&str>) -> Option<f64> {
    x?.parse().ok().filter(|v: &f64| v.is_finite())
}
fn endpoint(f: &[&str], id: usize) -> Option<LocalEndpoint> {
    Some(LocalEndpoint {
        endpoint_id: id,
        position: Position::new(finite(f.get(1))?, finite(f.get(2))?),
        direction: Angle::new(finite(f.get(3))?),
    })
}
fn straight(f: &[&str], idx: usize) -> Option<LocalTrackSegment> {
    Some(LocalTrackSegment::Straight {
        native_index: idx,
        start: Position::new(finite(f.get(3))?, finite(f.get(4))?),
        end: Position::new(finite(f.get(5))?, finite(f.get(6))?),
    })
}
fn curve(f: &[&str], idx: usize) -> Option<LocalTrackSegment> {
    let radius = finite(f.get(3))?;
    if radius == 0.0 {
        return None;
    }
    Some(LocalTrackSegment::Curve {
        native_index: idx,
        center: Position::new(finite(f.get(4))?, finite(f.get(5))?),
        radius: Distance::new(radius),
        start: Angle::new(finite(f.get(6))?),
        sweep: Angle::new(finite(f.get(7))?),
    })
}
fn route(f: &[&str]) -> Option<RouteState> {
    let name = f.get(1)?.trim_matches('"').to_owned();
    let mut traversals = vec![vec![]];
    for raw in &f[2..] {
        let n = raw.parse::<i64>().ok()?;
        if n == 0 {
            if traversals.last()?.is_empty() {
                return None;
            }
            traversals.push(vec![])
        } else {
            traversals.last_mut()?.push(RouteStep {
                native_segment_index: n.unsigned_abs() as usize,
                direction: if n > 0 {
                    "forward".into()
                } else {
                    "reverse".into()
                },
            })
        }
    }
    if traversals.last()?.is_empty() {
        return None;
    }
    Some(RouteState { name, traversals })
}
fn category(
    s: &[LocalTrackSegment],
    e: &[LocalEndpoint],
    r: &[RouteState],
    ok: bool,
) -> Option<String> {
    if !ok {
        return None;
    }
    if e.len() == 2 && s.len() == 1 {
        return Some(
            match s[0] {
                LocalTrackSegment::Straight { .. } => "straight",
                LocalTrackSegment::Curve { .. } => "curve",
            }
            .into(),
        );
    }
    if e.len() == 3 && r.len() >= 2 {
        return Some("turnout".into());
    }
    if e.len() == 4 && !r.is_empty() {
        return Some("crossing".into());
    }
    None
}
fn make_part(
    path: &Path,
    library: &str,
    ordinal: usize,
    line: usize,
    header: &str,
    group: Option<String>,
) -> Part {
    let title = header
        .split_once('"')
        .and_then(|(_, r)| r.rsplit_once('"').map(|(x, _)| x))
        .unwrap_or("")
        .to_owned();
    let before = header
        .split('"')
        .next()
        .unwrap_or("")
        .split_whitespace()
        .collect::<Vec<_>>();
    let scale = before.get(1).map(|x| x.to_string());
    let fields: Vec<_> = title.split('\t').collect();
    let (manufacturer, description, part_number) =
        if fields.len() == 3 || (fields.len() == 4 && fields[3].is_empty()) {
            (
                Some(fields[0].into()),
                Some(fields[1].into()),
                Some(fields[2].into()),
            )
        } else {
            (None, None, None)
        };
    let flags = before.get(2).and_then(|x| x.parse().ok());
    let fingerprint = digest(header.as_bytes());
    Part {
        part_id: id(
            "part",
            &[
                library.as_bytes(),
                ordinal.to_string().as_bytes(),
                header.as_bytes(),
            ],
        ),
        library_id: library.into(),
        fingerprint,
        source_ordinal: ordinal,
        location: Location {
            path: path.to_string_lossy().into(),
            line,
            end_line: line,
        },
        subcontents: group,
        native_type: "TURNOUT".into(),
        native_label: title,
        native_flags: flags,
        manufacturer,
        description,
        part_number,
        scale,
        gauge: None,
        category: None,
        category_basis: None,
        support_status: "unsupported".into(),
        geometry_complete: false,
        routes_complete: false,
        geometry: None,
        endpoints: None,
        routes: None,
        diagnostics: vec![],
        diagnostics_total: 0,
        diagnostics_truncated: false,
    }
}
