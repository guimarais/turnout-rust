use crate::{
    catalog::{Catalogue, PartFilter, SourceConfig},
    layout::{Diagnostic, Layout},
    xtc,
};
use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::BTreeSet,
    path::Path,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

struct InspectionSession {
    layout: Layout,
    catalogue: Catalogue,
}
pub struct InspectionServer {
    session: Mutex<Option<InspectionSession>>,
    config: SourceConfig,
    opens: AtomicU64,
    tool_router: ToolRouter<Self>,
}
impl Default for InspectionServer {
    fn default() -> Self {
        Self::new(SourceConfig::empty())
    }
}
impl InspectionServer {
    pub fn new(config: SourceConfig) -> Self {
        Self {
            session: Mutex::new(None),
            config,
            opens: AtomicU64::new(0),
            tool_router: Self::tool_router(),
        }
    }
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpenArgs {
    pub path: String,
}
#[derive(Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct PageArgs {
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelectArgs {
    pub library_ids: Vec<String>,
}
#[derive(Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct PartArgs {
    pub library_ids: Option<Vec<String>>,
    pub manufacturer: Option<String>,
    pub part_number: Option<String>,
    pub description: Option<String>,
    pub scale: Option<String>,
    pub category: Option<String>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetPartArgs {
    pub library_id: String,
    pub part_id: String,
}
fn state_error() -> ErrorData {
    ErrorData::internal_error("Inspection state is unavailable", None)
}
fn domain(d: Diagnostic) -> CallToolResult {
    CallToolResult::structured_error(json!({"ok":false,"diagnostics":[d]}))
}
fn no_layout() -> CallToolResult {
    domain(Diagnostic {
        severity: "error".into(),
        code: "no_open_layout".into(),
        message: "No layout is open".into(),
        location: None,
    })
}
fn info(layout: &Layout, catalogue: &Catalogue) -> CallToolResult {
    CallToolResult::structured(
        json!({"ok":true,"source":layout.source,"summary":layout.summary,"diagnostics":layout.diagnostics,"catalogue":catalogue.state()}),
    )
}
fn page(limit: Option<usize>) -> Result<usize, ErrorData> {
    let n = limit.unwrap_or(25);
    if !(1..=100).contains(&n) {
        Err(ErrorData::invalid_params(
            "limit must be from 1 through 100",
            None,
        ))
    } else {
        Ok(n)
    }
}
#[tool_router(router=tool_router)]
impl InspectionServer {
    #[tool(
        description = "Open a format-12 plain-text XTrackCAD .xtc layout for read-only inspection and refresh configured catalogue snapshots"
    )]
    async fn open_layout(&self, args: Parameters<OpenArgs>) -> Result<CallToolResult, ErrorData> {
        let mut state = self.session.lock().map_err(|_| state_error())?;
        *state = None;
        match xtc::read(Path::new(&args.0.path)) {
            Ok(layout) => {
                let catalogue = Catalogue::open(
                    &self.config,
                    &layout,
                    self.opens.fetch_add(1, Ordering::Relaxed) + 1,
                );
                let result = info(&layout, &catalogue);
                *state = Some(InspectionSession { layout, catalogue });
                Ok(result)
            }
            Err(error) => Ok(domain(error)),
        }
    }
    #[tool(
        description = "Return source, metadata, room dimensions, complete-or-unavailable track bounds, and catalogue state"
    )]
    async fn get_layout_info(&self) -> Result<CallToolResult, ErrorData> {
        let state = self.session.lock().map_err(|_| state_error())?;
        Ok(state
            .as_ref()
            .map_or_else(no_layout, |s| info(&s.layout, &s.catalogue)))
    }
    #[tool(
        description = "List supported placed tracks with explicit units, directions and endpoint references"
    )]
    async fn list_tracks(&self) -> Result<CallToolResult, ErrorData> {
        let state = self.session.lock().map_err(|_| state_error())?;
        Ok(match state.as_ref() {
            Some(s) => CallToolResult::structured(
                json!({"ok":true,"tracks":s.layout.tracks,"diagnostics":s.layout.diagnostics}),
            ),
            None => no_layout(),
        })
    }
    #[tool(
        description = "List all configured catalogue libraries from the immutable current layout snapshot"
    )]
    async fn list_track_libraries(
        &self,
        args: Parameters<PageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let n = page(args.0.limit)?;
        if args.0.cursor.is_some() {
            return Ok(domain(Diagnostic {
                severity: "error".into(),
                code: "invalid_cursor".into(),
                message: "Pagination cursors are not available for this request".into(),
                location: None,
            }));
        }
        let state = self.session.lock().map_err(|_| state_error())?;
        Ok(match state.as_ref() {
            Some(s) => {
                let all = s.catalogue.libraries();
                let total = all.len();
                CallToolResult::structured(
                    json!({"ok":true,"catalogue":s.catalogue.state(),"sources":s.catalogue.snapshot.sources,"libraries":all.into_iter().take(n).collect::<Vec<_>>(),"total":total,"next_cursor":serde_json::Value::Null,"diagnostics":[],"diagnostics_total":0,"diagnostics_truncated":false}),
                )
            }
            None => no_layout(),
        })
    }
    #[tool(
        description = "Atomically replace the current selected catalogue libraries; an empty list explicitly clears selection"
    )]
    async fn select_track_libraries(
        &self,
        args: Parameters<SelectArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        if args.0.library_ids.len() > 1024 {
            return Err(ErrorData::invalid_params(
                "library_ids may contain at most 1024 IDs",
                None,
            ));
        }
        let mut state = self.session.lock().map_err(|_| state_error())?;
        let Some(s) = state.as_mut() else {
            return Ok(no_layout());
        };
        match s.catalogue.select(args.0.library_ids) {
            Ok(()) => Ok(CallToolResult::structured(
                json!({"ok":true,"catalogue":s.catalogue.state(),"diagnostics":[],"diagnostics_total":0,"diagnostics_truncated":false}),
            )),
            Err(d) => Ok(domain(d)),
        }
    }
    #[tool(description = "List selected catalogue parts with optional metadata filters")]
    async fn list_track_parts(
        &self,
        args: Parameters<PartArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let a = args.0;
        let n = page(a.limit)?;
        if a.cursor.is_some() {
            return Ok(domain(Diagnostic {
                severity: "error".into(),
                code: "invalid_cursor".into(),
                message: "Pagination cursors are not available for this request".into(),
                location: None,
            }));
        }
        if [
            a.manufacturer.as_ref(),
            a.part_number.as_ref(),
            a.description.as_ref(),
            a.scale.as_ref(),
        ]
        .iter()
        .any(|x| x.is_some_and(|x| x.is_empty()))
        {
            return Err(ErrorData::invalid_params(
                "Text filters must not be empty",
                None,
            ));
        }
        let filter = PartFilter {
            library_ids: a
                .library_ids
                .map(|x| x.into_iter().collect::<BTreeSet<_>>()),
            manufacturer: a.manufacturer,
            part_number: a.part_number,
            description: a.description,
            scale: a.scale,
            category: a.category,
        };
        let state = self.session.lock().map_err(|_| state_error())?;
        let Some(s) = state.as_ref() else {
            return Ok(no_layout());
        };
        match s.catalogue.parts(&filter) {
            Ok(all) => {
                let total = all.len();
                Ok(CallToolResult::structured(
                    json!({"ok":true,"catalogue":s.catalogue.state(),"parts":all.into_iter().take(n).collect::<Vec<_>>(),"total":total,"next_cursor":serde_json::Value::Null,"diagnostics":[],"diagnostics_total":0,"diagnostics_truncated":false}),
                ))
            }
            Err(d) => Ok(domain(d)),
        }
    }
    #[tool(
        description = "Retrieve a selected catalogue part's local geometry, endpoints, routes, provenance, and completeness"
    )]
    async fn get_track_part(
        &self,
        args: Parameters<GetPartArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let state = self.session.lock().map_err(|_| state_error())?;
        let Some(s) = state.as_ref() else {
            return Ok(no_layout());
        };
        match s.catalogue.part(&args.0.library_id, &args.0.part_id) {
            Ok(part) => Ok(CallToolResult::structured(
                json!({"ok":true,"catalogue":s.catalogue.state(),"part":part,"diagnostics":[],"diagnostics_total":0,"diagnostics_truncated":false}),
            )),
            Err(d) => Ok(domain(d)),
        }
    }
}
#[tool_handler(router=self.tool_router)]
impl ServerHandler for InspectionServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo{instructions:Some("Read-only XTrackCAD format-12 inspection and immutable local catalogue browsing. Numeric geometry has explicit units; null values are unavailable rather than inferred.".into()),capabilities:ServerCapabilities::builder().enable_tools().build(),..Default::default()}
    }
}
