# Quickstart: Validate Feature 002

Status: future validation guide. This planning pass has not implemented or executed these new tools, fixtures, or test targets. The current implementation is still feature 001. Do not treat the commands or expected results below as passing evidence.

## Prerequisites

- Complete implementation derived from this plan and the later `tasks.md`.
- Existing Rust stable toolchain with rustfmt and Clippy; record actual Rust/Cargo versions.
- Local explicitly configured parameter sources; XTrackCAD does not need to run.
- An MCP client launching the binary over STDIO, or the actual executable integration harness.
- Authored fixture files and provenance manifest described below. Upstream sample data must not be vendored without documented terms.

## Planned fixture corpus

Create these during implementation under `tests/fixtures/catalog/`; retain GPL-3.0-only authorship/provenance, SHA-256, expected record counts, exact geometry/routes, and expected diagnostics in a manifest/README.

| Fixture group | Required content |
|---|---|
| `system-a.xtp`, `one-part.xtc` | Four track definitions: straight, curve, branching turnout, and crossing; layout uses only the straight |
| `system-b.xtp`, `mixed.xtc` | Different source with a colliding number and another scale; mixed-system layout |
| `duplicate-source.xtp`, duplicate records | Same structured identity/scale in distinct files and repeated records within one file |
| `empty.xtc`, custom-only layout | No matchable catalogue identities; explicit selection still works |
| Legacy/versioned grammar | Unversioned and PARAM 12; END/END$SEGS; labels/aliases/groups; legacy/versioned polygons before track slots; signed C/P references; concurrent zero-separated traversals |
| Partial/malformed grammar | Unsupported flags/adjustable parts, missing routes, invalid references, truncation, unknown framing, non-finite numbers, bad UTF-8, and unsupported PARAM versions |
| Generated source/query edge cases | Caps, empty/missing/unreadable sources, path spaces/Unicode, symlinks, duplicates, source mutation, page/filter changes, stale tokens, oversized/invalid ID arrays and ID strings, nested evidence/diagnostic budgets, and oversized mandatory result metadata |

Exact expected values must be independent of the parser under test. Include asymmetric geometry and decoration-before-track cases so swapped coordinates, lost radius signs, or filtered native indexes cannot accidentally pass.

## Build and configure after implementation

From the repository root:

```sh
cargo build --release --locked
```

Configure the client executable with separate arguments, for example:

```text
command: /absolute/path/to/turnout/target/release/turnout
args: ["--library-source", "/absolute/path/to/turnout/tests/fixtures/catalog/system-a.xtp"]
```

Use a working directory outside the repository in process tests. Repeat `--library-source` for another file/directory. No arguments must still launch the original inspection workflow with catalogue status `unconfigured`. See [configuration contract](contracts/configuration.md) for discovery and refresh rules.

## End-to-end client scenarios

1. Discover tools: the existing three plus `list_track_libraries`, `select_track_libraries`, `list_track_parts`, and `get_track_part`; verify schemas and no protocol stdout logs.
2. Open `one-part.xtc`. Verify unchanged source/summary fields plus catalogue state. An exact unique match selects system A automatically; evidence identifies the placed identity and the library without claiming original provenance.
3. List libraries, then list parts. Retrieve all four definitions, including the three unplaced ones. Compare exact source identity, raw UTF-8 fields, local coordinates, directions, native segment indexes, signed curves, route traversals, units, and completeness against the authored expectations.
4. Configure the duplicate source and reopen. Verify both candidates, distinct IDs, and no automatic selection. Explicitly select both, search by each supported filter, and page with limit 1 until every expected entry has appeared exactly once.
5. Submit an invalid replacement selection. Verify its failure preserves the prior selection and page tokens. Submit a valid replacement or clear selection; previous cursors must now fail and explicit clearing must not trigger automatic reselection.
6. Open empty/custom-only layouts, explicitly select system A, and retrieve its unplaced definitions. Without selection, part tools return `catalogue_not_selected`; the library list remains accessible.
7. Repeat valid layout inspection with no sources, a missing file, unreadable directory, invalid parameter encoding, malformed definitions, and supported/unsupported mixtures. Verify layout success, source-located catalogue diagnostics, safe recovery, and complete-versus-partial distinctions. No incomplete geometry array may carry a complete flag.
8. Change a library's bytes after open. Its current detail/fingerprint must remain the captured snapshot. Reopen and verify updated content/fingerprint and fresh resolution; old cursors are stale. Delete a source after open and confirm current snapshot remains inspectable until reopen.
9. Successfully open a layout, then fail another open. All existing and new session tools must return `no_open_layout`. Open a different valid layout and confirm no previous explicit choices or evidence survive.
10. Before/after every successful, failed, and partial workflow, compare source bytes and modification times. Existing feature 001 version, geometry, unavailable-bound, UTF-8, and failure cases must retain their behaviour.

MCP payloads and matching rules are authoritative in [the tool contract](contracts/mcp-tools.md). Use the existing ten-second handshake/request/shutdown timeouts and child-exit/cleanup assertions. Test missing/unreadable paths under permissions that actually enforce denial; document conditional skips rather than claiming coverage from a privileged account.

## Pinned real-library check

Use a separately supplied local copy of [`mrklnhoc-de.xtp` at revision 6058992cb3a97d97ba373dfcfb7f17ccc52aef15](https://github.com/adbyrne/XTrkCAD/blob/6058992cb3a97d97ba373dfcfb7f17ccc52aef15/app/lib/params/mrklnhoc-de.xtp). Keep it outside the committed corpus unless its data redistribution terms and notices are recorded.

Verify SHA-256 `d76b90fd573e7b7d33b9fd806fb338f8097e081b268fd9ad2d729b3c6b6611c5` using the local platform's checksum utility. Configure that file and open `layouts/mrkln_sample.xtc`. Required outcomes:

- Enumerate 43 track entries, without requiring an absent PARAM directive or rejecting legacy END/F framing.
- Match the three distinct placed identities 24229, 24130, and 24612 across four placed instances; retain UTF-8 titles and source-specific IDs.
- Retrieve complete supported straight, curve, basic turnout, and crossing details; in particular exercise the crossing's `1 0 2` concurrent path state.
- Keep the flagged double slip visible with unsupported flag/completeness diagnostics; no demand that all 43 entries be complete under the bounded subset.
- Confirm unchanged source bytes/mtime and unchanged embedded layout geometry.

Record source revision, checksum, environment, exact results, and unsupported records in verification notes during implementation. This research-selected target is not an already validated parser fixture. Ordinary automated tests use authored data and do not download this source.

## Planned automated checks and final gate

The new targets below must be authored during implementation; they do not exist as part of this plan.

```sh
cargo test --locked --test xtp_tests
cargo test --locked --test catalog_tests
cargo test --locked --test config_tests
cargo test --locked --test mcp_stdio_tests
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
rustc --version
cargo --version
```

Retain all 42 baseline test scenarios, adapting only additive tool-discovery expectations. Add new exact-value tests and real STDIO scenarios; do not substitute only in-memory protocol tests. Record actual totals and compiler versions only after running the implemented suite. README/client setup, source limits, supported syntax, refresh, and added dependency attribution must then be updated to reflect actual behaviour.

## Requirement and acceptance coverage

| Requirement | Validation |
|---|---|
| FR-001 | Explicit files/directories, no-config launch, path/duplicate/symlink tests, no unconfigured reads |
| FR-002 / FR-004 | Unique, mixed, ambiguous, unmatched, and incomplete-inventory resolution; evidence/status/origin flags |
| FR-003 | Empty/custom layout selection, multiple sources, atomic invalid replacement, explicit clear |
| FR-005 / FR-007 | Four-part catalogue, unplaced entries, duplicates, stable IDs and exact source fingerprints |
| FR-006 | Every filter, page-size bounds, deterministic complete traversal, stale/invalid tokens and response bounds |
| FR-008 / FR-009 | Exact local geometry/endpoints/routes, signs/indexes, UTF-8, conservative categories, null gauge |
| FR-010 | Recoverable malformed/unsupported entries and unrecoverable boundary, source locations and completeness |
| FR-011 / FR-016 | Existing layout semantics under every source failure; old fields/inputs intact |
| FR-012 / FR-013 | New/failed open clearing, selection revisions, source mutation and reopen refresh |
| FR-014 | Source byte/mtime checks across all input/outcome classes |
| FR-015 | Every discovery/input/definition/diagnostic/evidence/page/response cap, explicit truncation and coverage |
| FR-017 | Authored manifests, Linux checks, pinned real-source run, real executable lifecycle |
| SC-001 / SC-002 | Four-part workflow and mixed/duplicate/ambiguous fixtures |
| SC-003 / SC-004 | Failure/completeness matrix and all selection/query lifecycle cases |
| SC-005 / SC-006 | Source integrity, 001 regression scenarios, full STDIO workflow |

All seventeen user-story acceptance scenarios are covered by the end-to-end sequences and the corresponding fixture/test groups. Completion requires evidence for every row, not checked boxes in this planning document.
