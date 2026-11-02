# Data Model: Track Parameter Catalogue Retrieval

All types are application-owned. XTP records are mapped before returning results. Existing placed-layout types and world coordinates remain unchanged. Optional values use explicit JSON null; known empty collections use arrays, not null.

## SourceConfig and CatalogueSnapshot

`SourceConfig` contains ordered startup paths and captured startup working directory. It outlives the session; it is not editable through MCP in 002.

`CatalogueSnapshot` owns `snapshot_id`, ordered source statuses, library snapshots, part indexes, inventory coverage, and diagnostics. It is created after a valid layout parse on every open. All bytes/fingerprints/definitions come from the same bounded reads. Detail requests never reread files.

| SourceStatus field | Rules |
|---|---|
| `configured_path`, `resolved_path` | Original and absolute source display paths; preserve internal OS path identity |
| `kind` | `file`, `directory`, or null when unavailable |
| `availability` | `ready`, `partial`, or `unavailable` |
| `diagnostics`, `diagnostics_total`, `diagnostics_truncated` | Bounded source issues and explicit retained/total distinction |

Directory status is distinct from a library: an unreadable directory is still reported even though its children cannot be enumerated.

## LibrarySnapshot / LibrarySummary

| Field | Rules |
|---|---|
| `library_id` | `lib:sha256:<hex>` over versioned canonical source identity; independent of catalogue labels |
| `path` | Absolute canonical source display path; unresolved normalized absolute path for unavailable explicit files |
| `fingerprint` | `sha256:<hex>` over exact file bytes, or null if unavailable/limit prevents a full read |
| `contents`, `aliases` | First label and subsequent labels in source order; null/empty when not readable |
| `parameter_version` | Explicit accepted native `PARAM` value, or null for an unversioned file |
| `availability` | `ready`, `partial`, or `unavailable` |
| `inventory_complete` | True only if all configured file content was safely enumerated; not a geometry support claim |
| `part_count`, `complete_part_count`, `incomplete_part_count` | Counts over indexed track entries; cannot stand in for an incomplete inventory total |
| `selected` | Current selection membership, supplied in library tool responses |
| `evidence`, `evidence_total`, `evidence_truncated` | Up to 20 matching evidence samples; each has at most 20 part IDs and 20 placed track IDs with separate total/truncated fields |
| `diagnostics`, `diagnostics_total`, `diagnostics_truncated` | Source/definition issues, bounded as contracted |

`availability: partial` permits selection when safe entries or metadata exist. `unavailable` does not. A library can have `inventory_complete: true` and incomplete definitions: every boundary is known even though some geometry is unsupported. Incomplete discovery of any configured source makes the global inventory incomplete.

## PartDefinition / PartSummary

| Field | Rules |
|---|---|
| `part_id` | `part:sha256:<hex>` over library ID, source record ordinal, and exact record fingerprint |
| `library_id`, `fingerprint` | Owning source ID and `sha256:` digest of the exact definition span |
| `source_ordinal`, `location` | One-based ordinal among all recognized top-level object records and `{path,line,end_line}`; duplicates remain distinct |
| `subcontents` | Active group label, or null |
| `native_type`, `native_label`, `native_flags` | Preserve `TURNOUT`, complete quoted title, and flags where present |
| `manufacturer`, `description`, `part_number`, `scale` | Parsed supported fields, each nullable; raw label is never replaced by normalized search text |
| `gauge` | Explicit `{value,unit}` only if supported source data declares one; otherwise null. Initial header grammar provides none |
| `category` | `straight`, `curve`, `turnout`, `crossing`, or null, under the reader rules |
| `category_basis` | `geometry_and_routes` or null |
| `support_status` | `complete`, `partial`, or `unsupported`, within this feature's track-definition subset |
| `geometry_complete`, `routes_complete` | Separate booleans; do not imply placement or whole-file support |
| `geometry`, `endpoints`, `routes` | Detail-only arrays or null when the respective value cannot be interpreted completely |
| `diagnostics`, `diagnostics_total`, `diagnostics_truncated` | Definition-local issues; always expose completeness even if messages are truncated |

Listings return the metadata/completeness fields but omit detail-only geometry/endpoints/routes. A detail result for incomplete data returns trustworthy metadata, false completeness, and null for each incomplete component, rather than publishing a partial array as complete. An absent native route becomes `routes: null`, `routes_complete: false`, with `routes_unavailable`; it is not an invented route or an empty fully known route set.

`complete` requires supported track semantics, finite geometry, at least two reliable endpoints, and valid routes. `partial` has inspectable metadata/geometry but an unsupported or invalid component. `unsupported` identifies a track entry without usable supported geometry. Decorations are framed for indexing but not exported as track geometry; their deliberate omission alone is not incomplete track support.

## Geometry, endpoints, and routes

- Geometry is library-local XY in inches, no placement transform. Zero degrees points along positive Y and increases clockwise. Reuse owned `Position`, `Angle`, and `Distance` shapes from 001 where valid.
- `LocalTrackSegment` has `native_index` (one-based full drawable sequence), `kind`, and the understood straight or curve fields. A straight exposes start/end positions. A curve exposes center, signed radius, start angle, and sweep. Every number is finite.
- The internal native segment table also retains decorative and unsupported slots. A polygon is one slot regardless of vertex count. Exported track segments may have nonconsecutive native indexes.
- `LocalEndpoint` has one-based `endpoint_id`, position, and outward direction. It has no placed-track `connected_to` or open/connected state.
- `RouteState` contains its native name and `traversals`. Each traversal contains ordered `{native_segment_index, direction}` steps and matching `start_endpoint_id`/`end_endpoint_id` when established.
- `direction` is `forward` for positive native references and `reverse` for negative references, relative to the native segment's endpoint ordering. Curve radius sign and path reference sign are preserved independently.
- Position equality for route validation uses `1e-5` inch, matching the existing reader's tolerance for six-decimal native rounding. No connection solver or inferred physical compatibility is supplied.

## Identity and provenance

Hash inputs use a versioned domain prefix and length-prefixed byte fields, not ambiguous string concatenation. Source identity hashes canonical OS path bytes, retaining non-UTF-8 distinctions; display paths are informational. Existing aliases to the same canonical file deduplicate. Different files with identical content remain distinct libraries. Hard links at different canonical paths are treated as distinct sources, so a matching pair remains ambiguous.

Library IDs remain stable when content changes at the same canonical path, while fingerprints change. Part IDs remain stable only when library identity, record ordinal, and record bytes are unchanged. Moving a file or inserting earlier object records can change IDs. These are reproducible source identities on the same platform, not universal manufacturer product IDs. Do not alter 001's existing `fnv1a64:` layout fingerprint.

## Resolution and CatalogueState

For placed tracks with structured labels, use the exact `(scale, manufacturer, description, part_number)` tuple. Preserve Unicode, case, and field contents; remove only native delimiters and an optional trailing empty tab field. Missing/unsupported tuples are unmatched, not fuzzy guesses. Candidate evidence contains the tuple, matching part/library IDs, and placed track IDs; bounded samples carry total/truncated fields.

Compute library candidates per tuple. Automatic selection is allowed only when global source enumeration is complete, at least one tuple matches, and every matching tuple resolves to exactly one distinct library. Select the union of those libraries. Any competing library suppresses all automatic selection for this open; unmatched instances remain counted separately and never imply that the selected set covers all layout systems. Repeated definitions within one library retain distinct matching part IDs; library selection is not an exact definition choice.

| CatalogueState field | Rules |
|---|---|
| `session_id`, `snapshot_id` | Current open session and immutable catalogue snapshot |
| `status` | `unconfigured`, `no_match`, `candidates`, `unselected`, or `selected` |
| `selection_origin` | `automatic`, `explicit`, or null |
| `selection_revision` | Monotonic within session; increments on every successful explicit selection, including clearing |
| `selected_library_ids` | Sorted, distinct selected IDs |
| `inventory_complete` | Global enumeration guarantee; false after unreadable sources, unsafe framing, or discovery/parser limits |
| `has_incomplete_definitions` | Any indexed selected entry has non-complete support |
| `counts` | Configured sources, indexed libraries/parts, matched/unmatched/ambiguous placed-track counts |
| `diagnostics`, `diagnostics_total`, `diagnostics_truncated` | Catalogue-specific, separate from layout diagnostics |

Empty startup configuration initially yields `unconfigured` after open. With sources but no available matches, use `no_match`. Matches blocked by ambiguity or incomplete enumeration yield `candidates`. Successful explicit empty selection yields `unselected` with explicit origin even when configuration is empty; the explicit choice overrides the initial resolution status until the next open. Any nonempty valid selection yields `selected`, with inventory and support flags retaining limitations.

## Session transitions and query continuation

```text
Process config persists
  open_layout starts → clear entire prior session and its tokens
    layout fails → no current session
    layout succeeds → fresh catalogue snapshot → resolution → new current session
  valid explicit selection → atomically replace set; increment selection revision
  invalid selection → preserve set and revision
  list/detail → immutable snapshot; never reread sources
```

One mutex protects `Option<InspectionSession>`, containing the layout, catalogue snapshot, evidence, and selection. Serialize open and selection mutations; no await while holding the guard. The initial implementation may perform bounded synchronous loading under this serialization, consistent with 001. Do not introduce concurrent/background refresh paths.

Generate a process nonce using OS randomness once; pair it with a monotonic successful-open/session counter to prevent cross-open/restart token reuse. A nonce initialization failure is an internal startup error on stderr. Snapshot ID hashes the ordered file identities/fingerprints/statuses. Cursors encode a version, session ID, selection revision, query kind/filter digest, and offset. Cursor state is opaque continuation data, not authorization; no signing or persistence is required. See [MCP contract](contracts/mcp-tools.md).
