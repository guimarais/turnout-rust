# MCP Contract: Catalogue Retrieval

Status: planned; all seven tools use the existing STDIO connection and locked RMCP transport. The four catalogue tools below are new. [Data model](../data-model.md) defines all response entities; [configuration](configuration.md) defines source and payload bounds.

## Shared rules

- Successful tool results use structured JSON with `ok: true` and equivalent JSON text. Expected domain failures use structured error results with `ok: false`, `isError: true`, and diagnostics. Unknown fields, missing required arguments, wrong types, invalid enum values, and documented argument-length/range violations are MCP `invalid_params` errors.
- Every new successful catalogue result includes `catalogue` (the current CatalogueState). All optional fields use explicit null. Arrays with known zero elements are empty arrays. Never serialize non-finite numbers.
- All four new tools require a successfully opened layout, including browsing configured libraries for an empty layout. Without one, return `no_open_layout`; do not expose prior session entries or selections.
- Every input library/part ID must match `lib:sha256:` or `part:sha256:` followed by exactly 64 lowercase hexadecimal digits, respectively. Other ID syntax is `invalid_params`; well-formed IDs absent from the current snapshot are domain failures. Input ID arrays contain at most 1,024 entries before deduplication and normalize to sorted distinct IDs.
- Catalogue diagnostics remain separate from existing layout diagnostics. A valid layout has `ok: true` even when catalogue sources are unavailable or ambiguous. Catalogue messages/counters/flags must make that limitation visible.
- Catalogue selection does not modify any layout object, declared room, track bound, embedded definition, or file. It does not validate that selected parts physically connect.
- Apply the combined diagnostic/evidence and result-size budgets in [configuration](configuration.md), preserving total/truncated fields. New tool payloads are bounded; existing 001 layout payloads are not newly capped. Rendering catalogue limitations must preserve successful layout opening.

## Existing `open_layout` and `get_layout_info`

Inputs and existing `ok`, `source`, `summary`, and `diagnostics` fields retain their 001 meanings. Add only a top-level `catalogue` field to successful results. `list_tracks` keeps its existing fields and behaviour; no catalogue data is inserted into track objects.

On `open_layout`, first invalidate the entire old session. Parse the layout using unchanged format-12 rules. Only a valid layout triggers bounded library discovery, snapshot capture, and resolution. Publish the new layout and catalogue coherently. Failure leaves no current session. Reopening always creates a fresh session ID, snapshot, and selection, even for unchanged files; content-derived library/part IDs may remain stable.

Illustrative catalogue state with no configured sources:

```json
{
  "session_id": "opaque-session",
  "snapshot_id": "sha256:<digest>",
  "status": "unconfigured",
  "selection_origin": null,
  "selection_revision": 0,
  "selected_library_ids": [],
  "inventory_complete": true,
  "has_incomplete_definitions": false,
  "counts": {
    "sources": 0, "libraries": 0, "parts": 0,
    "matched_tracks": 0, "unmatched_tracks": 4, "ambiguous_tracks": 0
  },
  "diagnostics": [],
  "diagnostics_total": 0,
  "diagnostics_truncated": false
}
```

Digest placeholders are illustrative, not literal valid IDs. With no configured sources, empty inventory enumeration is complete; it does not mean libraries for the layout have been found. `get_layout_info` reports the current selection after explicit changes without rereading files.

## `list_track_libraries`

Input object with optional `cursor: string | null` and `limit: integer` (default 25, range 1–100). Empty `{}` is valid.

Result fields:

| Field | Meaning |
|---|---|
| `ok`, `catalogue` | Success and current session state |
| `sources` | Up to 64 configured SourceStatus summaries, including directory failures |
| `libraries` | One bounded page of LibrarySummary entries, including unavailable explicit files |
| `total` | Indexed library count, not an estimate of undiscovered files |
| `next_cursor` | Continuation string or null |
| `diagnostics`, `diagnostics_total`, `diagnostics_truncated` | Request-level issues; catalogue/source issues stay in their entities |

List all indexed configured libraries, not only matches or selected libraries. Include source identity, fingerprint, labels, availability, support counts, selection membership, bounded tuple/placed-track matching evidence, and its total/truncated flags. No configuration yields a successful empty list with `unconfigured` state. No automatic source discovery beyond configured locations occurs.

## `select_track_libraries`

Required input: `{"library_ids":["lib:sha256:<digest>"]}`. Maximum 1,024 IDs; duplicate IDs are deduplicated. Each ID must refer to an available current-snapshot library. `partial` libraries are selectable with explicit limitations; `unavailable` libraries are rejected.

Validate the entire request before mutating state. Replace, rather than append to, the selection atomically; sort IDs; set origin `explicit`; increment selection revision. Return `ok`, updated `catalogue`, and request diagnostics. Repeating an identical successful selection still increments the revision, intentionally invalidating earlier cursors. Failure preserves selection and revision.

An empty array explicitly clears selection, producing status `unselected`, origin `explicit`, and no selected IDs. Do not rerun automatic selection until another layout open.

## `list_track_parts`

All fields optional, unknown fields rejected:

```json
{
  "library_ids": null,
  "manufacturer": null,
  "part_number": null,
  "description": null,
  "scale": null,
  "category": null,
  "cursor": null,
  "limit": 25
}
```

- Null/omitted `library_ids` means all selected libraries. A supplied nonempty array (maximum 1,024 entries, deduplicated/sorted) narrows to selected IDs; unknown or unselected IDs fail instead of expanding selection. An empty supplied array is invalid.
- Filters combine with AND. Manufacturer, number, and description use Unicode lowercase substring matching, without accent folding, locale rules, or whitespace trimming. Empty text filters are invalid; native labels themselves remain verbatim.
- Scale uses Unicode lowercase exact matching without whitespace trimming. Category is the exact enum `straight`, `curve`, `turnout`, `crossing`, or `unknown`; `unknown` selects null categories. Missing metadata does not satisfy a supplied text/scale filter.
- Return all indexed matching entries, including partial/unsupported definitions; do not silently hide them. Summaries omit geometry arrays and identify support/completeness.
- No selection returns `catalogue_not_selected`. A selected library with zero entries returns a successful empty page. No matches is also successful.

Result contains `ok`, `catalogue`, `parts`, `total` (indexed matches), `next_cursor`, and request-level diagnostic fields. Order is library ID byte order, then native source ordinal. Duplicate identities remain separate entries. `total` describes indexed entries only; `catalogue.inventory_complete` conveys whether additional undiscovered content may exist.

## `get_track_part`

Required input: `{"library_id":"lib:sha256:<digest>","part_id":"part:sha256:<digest>"}`.

The library must exist in the current snapshot and be selected, and the part must belong to it. Return `ok`, `catalogue`, `part` (PartDefinition), and request diagnostics. Complete data is in local inches/clockwise degrees with source-native segment indexes and named route states. Return metadata and completeness for an unsupported entry with `ok: true`; missing geometry/routes are null with definition diagnostics. This is a successful inspection of an unsupported definition, not a claim of supported placement.

Never replace placed layout geometry with the returned definition. Snapshot data remains available if its source is changed/deleted after opening; only reopening refreshes it. A changed ID that is not in the current selected snapshot fails explicitly rather than returning old data.

## Pagination and continuation

- Page both library and part lists. Default 25, maximum 100. Responses may be shorter than the requested limit to obey the JSON byte cap; `next_cursor` continues after the last returned entry. An entry that cannot fit even alone returns an explicit response-limit failure, never a silent skip.
- Cursors are opaque versioned ASCII strings no longer than 512 bytes containing session ID, selection revision, query kind/filter digest, and next offset. No raw path is required. They are continuation state, not access credentials.
- Canonicalize filters in a fixed field order, null defaults, normalized matching values, and sorted/deduplicated library subset before hashing. Bind page size too: subsequent requests must repeat the same filters/subset/limit. Use offset after the last returned entry, not requested page size.
- Reject malformed cursor encoding/version/out-of-range offsets with `invalid_cursor`. Reject a well-formed cursor belonging to another session/revision/query with `stale_cursor`. Library cursors are also selection-revision bound because their selected flags can change.
- Reopen, failed open, server restart, or any successful explicit selection invalidates old cursors. Failed selection does not. Zero current session takes precedence and returns `no_open_layout`.

## Diagnostic codes and placement

| Code | Meaning / outcome |
|---|---|
| `no_open_layout` | No usable session; expected tool failure |
| `catalogue_not_selected` | No current library selection; expected tool failure |
| `library_not_found`, `library_unavailable`, `library_not_selected` | Invalid domain selection or detail/subset target; no mutation |
| `part_not_found` | Part absent from the specified current library |
| `invalid_cursor`, `stale_cursor` | Invalid continuation; no mutation |
| `source_unavailable`, `source_skipped`, `source_changed_during_read` | Catalogue source warning; layout remains usable |
| `unsupported_parameter_version`, `malformed_parameter_file`, `invalid_parameter_utf8` | Source unavailable or partial as defined by safe framing |
| `malformed_definition`, `unsupported_definition`, `unsupported_parameter_record` | Entry/source limitation, location and completeness required |
| `invalid_route`, `routes_unavailable` | Routes incomplete; preserve independently complete geometry |
| `catalog_limit_exceeded`, `response_limit_exceeded` | Explicit incomplete coverage/response; never a false complete array |

Diagnostic structure preserves 001's `severity`, `code`, `message`, and optional `{path,line}` location. Catalogue entities additionally identify affected library/part and end line through their own provenance fields. Library/definition parsing errors are nested catalogue limitations even if severity is `error`; they do not set a valid layout result's top-level `ok` false. Code and severity are stable; prose is not. Bounded arrays expose `diagnostics_total` and `diagnostics_truncated`.

Internal mutex/nonce failures are MCP internal errors, not ordinary source absence. Do not emit logs on protocol stdout.
