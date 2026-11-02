# Contract: Catalogue Sources and Resource Bounds

Status: planned interface, not implemented. Source requirements: FR-001, FR-012–FR-015.

## Startup interface

```text
turnout [--library-source PATH]...
```

- No arguments retains 001 startup behaviour and gives catalogue status `unconfigured` after a successful layout open.
- Repeat the flag for a regular `.xtp` file or a directory. Extension matching is ASCII case-insensitive. A missing path is retained as an unavailable source; it does not stop the MCP server.
- Parse OS arguments as `PathBuf`, not colon-separated strings. Paths containing spaces are one client argument. Resolve relative paths against the captured startup working directory; recommend absolute paths in client setup. No tilde, glob, or environment expansion occurs inside Turnout.
- Unknown arguments, missing/empty flag values, or more than 64 configured entries fail startup with a nonzero exit and a concise stderr diagnostic, without writing protocol data to stdout.
- No implicit platform directory search, environment-variable configuration, config-file precedence, XTrackCAD configuration import, network retrieval, or runtime source reconfiguration in 002.

Example MCP client configuration after implementation:

```json
{
  "mcpServers": {
    "turnout": {
      "command": "/absolute/path/to/turnout",
      "args": ["--library-source", "/absolute/path/to/track libraries"]
    }
  }
}
```

## Discovery and snapshot rules

1. After a valid layout parse, resolve the configured sources and inspect direct directory children only. Do not recurse. Ignore ordinary non-XTP files and subdirectories.
2. Explicitly configured symlinks may resolve to a regular file or directory. During directory scans, do not follow child symlinks; report skipped `.xtp` symlink entries as `source_skipped` and mark source enumeration incomplete. Users can configure the intended target explicitly.
3. Canonicalize existing files and deduplicate canonical paths before parsing. Preserve the originating configured-source associations. Different files, including hard links with distinct canonical paths, remain distinct sources. Non-UTF-8 paths retain OS-native identity internally even if display strings require replacement characters.
4. Enumerate within the entry cap, then sort canonical path identities by their native encoded byte order before parsing or applying aggregate file/byte budgets. Ordering is deterministic for a fixed filesystem/platform. If enumeration itself reaches its cap, discard that directory's incomplete discovered set, report it unavailable, and keep other bounded sources; never imply the unsorted prefix is a complete inventory.
5. Use bounded reads, not metadata size alone: read at most the per-file budget plus one byte, check aggregate bytes actually read, and refuse oversized/non-regular data. Detect file replacement or size/mtime change across the read and mark the file unavailable with `source_changed_during_read`. Fingerprints describe the captured bytes, not an assertion that the live file will remain unchanged.
6. Parse exact captured bytes; keep source bytes and definitions together. All catalogue browsing uses the snapshot without disk I/O. Edits/deletions after open do not change active results. Reopen the same or another layout to refresh all configured sources; there is no watcher or refresh tool.
7. Missing sources, permission errors, invalid encoding, and malformed libraries produce catalogue diagnostics and coverage flags, never failure of an otherwise valid layout. Configuration remains outside session state. A failed layout open clears session-specific results before any catalogue refresh.

## Initial bounds

These fixed defaults are planning choices, not benchmarks or native-format limits. They are not configurable in 002.

| Area | Bound | Behaviour at limit |
|---|---|---|
| Configured sources | 64 | Invalid startup configuration |
| Directory entries examined | 16,384 total per open, including unrelated entries | Discard incomplete current-directory enumeration; diagnose and mark global coverage incomplete |
| Distinct discovered parameter files | 1,024 | Retain sorted bounded inventory, diagnose omitted files and incomplete coverage |
| File bytes | 8 MiB | File unavailable with `catalog_limit_exceeded` |
| Aggregate source bytes | 64 MiB per open | Stop further file reads; affected sources unavailable and global coverage incomplete |
| Recognized top-level object records | 100,000 per snapshot | Stop indexing beyond cap; diagnose incomplete enumeration |
| Single definition bytes | 256 KiB | Preserve recoverable metadata and mark definition unsupported/incomplete; recover only at a proven boundary |
| Drawable slots per definition | 4,096 | Incomplete definition with diagnostic |
| Endpoints / named route states | 256 each per definition | Incomplete respective component |
| Path references | 4,096 total per definition | Incomplete routes |
| Label/metadata field bytes | 4 KiB per field | Do not truncate into a seemingly exact label; expose unavailable field with an explicit limit diagnostic |
| Diagnostic messages | 100 retained per source/library/catalogue, 10 per part; 1 KiB per message | Report emitted issue total and retained/truncated status; message truncation never changes support/coverage flags |
| Evidence samples | 20 per library; at most 20 matching part IDs and 20 placed track IDs per sample | Report total/truncated counts at every level; include the matching tuple |
| All rendered diagnostic arrays combined | 64 KiB JSON bytes per response | Trim retained messages deterministically with original total/truncated fields; never drop coverage/support flags |
| Page size | Default 25, maximum 100 | Invalid arguments outside 1–100 |
| Filter / cursor bytes | 256 per text filter / 512 per cursor | Invalid arguments |
| New catalogue tool structured payload / additive `catalogue` field | 1 MiB JSON bytes each | Apply the response-budget policy below; existing 001 payloads retain their contract |

The structured object and equivalent JSON text each contain the same bounded payload; MCP envelope/duplicate representation overhead is additional. The cap applies to new catalogue tool results and, independently, the additive `catalogue` object on successful open/info. It does not impose a new limit on 001's existing layout/track results, and catalogue rendering must never fail an otherwise valid layout open.

Before applying page-size reduction, render diagnostics under their combined 64 KiB budget and trim optional evidence samples/ID arrays as necessary, preserving total/truncated markers. Do not drop mandatory configured-source summaries to fit a library page. If mandatory metadata alone cannot fit in a new tool response even with empty diagnostics/evidence, return a compact `response_limit_exceeded` domain failure with no oversized location fields. Open/info can always fall back to compact catalogue status, IDs, coverage flags, counts, and diagnostic truncation markers while preserving layout success.

Definition completeness reflects the parsed snapshot. If a response-size cap prevents transmitting a complete component, a detail response retains metadata, nulls the affected arrays, sets their response completeness false, and reports `response_limit_exceeded`; it never publishes a truncated array with a complete flag.

Source/record limits affect `inventory_complete` when enumeration is unfinished. A fully framed but unsupported definition affects its support status, not necessarily inventory coverage. Automatic selection requires globally complete enumeration even if some enumerated definitions have incomplete geometry. Diagnostic totals count issues detected before stopping, not unknown issues in unread regions.

Implementation must verify file/record/segment/diagnostic bounds with generated test inputs and avoid allocating from an unchecked declared count. Filesystem calls themselves have no hard real-time guarantee; the ten-second request timeout is a test harness boundary, not a claim to interrupt every stalled filesystem operation.
