# Implementation Plan: Track Parameter Catalogue Retrieval

**Feature**: `002-track-parameter-catalog` | **Date**: 2026-10-31 | **Spec**: [spec.md](spec.md)

**Input**: `specs/002-track-parameter-catalog/spec.md`

**Workspace note**: Planning is performed on the existing `001-readonly-layout-inspection` Git checkout. Spec-Kit feature identity and Git branch are independent in this installation. No branch, commit, PR, application code, dependency, or test implementation is created by this planning pass. The local `.specify/feature.json` selects 002 for subsequent commands.

## Summary

Extend the existing Rust STDIO MCP server with read-only track parameter catalogue retrieval. At each successful layout open, discover only explicitly configured `.xtp` sources, capture bounded immutable snapshots, parse supported reusable definitions, and resolve candidate libraries from placed identity and scale. Expose four catalogue tools with explicit selection, filtering, pagination, provenance, and completeness. Preserve the three feature 001 tools and all existing placed geometry.

The first real compatibility target is the pinned German Märklin C-Gleis library described in [research.md](research.md), matching the supplied layout's part labels. Independently authored fixtures establish exact behaviour without redistributing upstream data.

## Technical Context

**Language/Version**: Existing Rust 2024 project, rolling stable toolchain. Record actual compiler versions during implementation validation.

**Primary Dependencies**: Retain locked `rmcp` 0.8.5, Tokio, Serde, `serde_json`, and `schemars`. Plan direct `sha2` 0.11 for versioned SHA-256 identities and `getrandom` 0.4 for a process nonce used in session tokens. Implementation updates `Cargo.lock` and attribution after resolving their graph; this plan changes neither.

**Storage**: No database, cache files, or source writes. Process-lifetime configuration and one in-memory layout/catalogue session. Snapshots retain exact bounded source bytes and parsed records together.

**Testing**: Existing 42-test baseline plus new native-reader, catalogue/query, configuration, and real-executable STDIO integration tests. Validation scenarios are specified in [quickstart.md](quickstart.md); none are claimed to have run for 002.

**Target Platform**: Linux first; portable `PathBuf`/OS arguments and filesystem APIs for eventual Windows/macOS support. No production shell commands.

**Project Type**: Existing single executable and library core; no new service or transport.

**Performance Goals**: Deterministic bounded discovery/parsing and bounded responses. Initial operational caps are specified in [configuration.md](contracts/configuration.md); they are implementation guardrails, not measured performance promises. All fixture requests must finish within the existing ten-second integration timeout.

**Constraints**: Explicit local sources only; immutable browsing snapshot; source integrity; no `INCLUDE` traversal; no guessing provenance, gauge, or geometry. Unsupported definitions remain visible with completeness. Preserve structured results and equivalent JSON text.

**Scale/Scope**: One active layout session, up to 64 configured sources and 1,024 discovered files, with bounded bytes/records. Support unversioned legacy `.xtp` and explicit `PARAM 12` within the defined grammar; `.xtp` `PARAM` and `.xtc` `VERSION` are separate compatibility policies. Seven MCP tools total after implementation.

## Constitution Check

The constitution remains an unfilled template, so no ratified project-specific MUST rules can be inferred from it. Apply the explicit project and user constraints instead.

| Gate | Before research | After design | Evidence |
|---|---|---|---|
| Documentation only in this pass | PASS | PASS | Artifacts limited to this feature and local Spec-Kit selection |
| Immutable sources and no editing | PASS | PASS | Snapshot reader; no write/placement interface |
| Evidence-based native semantics | PASS | PASS | Pinned source research, bounded grammar, incomplete states |
| Owned model and adapter isolation | PASS | PASS | Separate catalogue and XTP modules; placed model unchanged |
| 001 compatibility | PASS | PASS | Additive catalogue field and four tools; old tool semantics retained |
| Portable paths, Linux validation first | PASS | PASS | OS arguments, shallow explicit discovery, no shell |
| Fixture/dependency provenance | PASS | PASS | Authored fixtures; external library referenced, not copied; attribution gate |

No gate violations or unresolved product decisions remain. These are design assessments; implementation validation remains outstanding.

## Phase 0 — Documentation analysis and research

[research.md](research.md) records the full documentation inventory, resolved choices, alternatives, and primary sources. Three research agents separately reviewed native XTP semantics, existing Rust/MCP integration, and all product documentation.

Important findings: catalogue `TURNOUT` is not a physical category; XTP accepts legacy files without `PARAM`; routes reference the full drawable-segment array, including decorative slots; signed indexes control traversal, and zero separates concurrent subpaths. Reusing the XTC layout parser or its format-12 gate directly would be incorrect.

## Phase 1 — Design and contracts

- [data-model.md](data-model.md): owned entities, identities, resolution, completeness, and session transitions.
- [contracts/configuration.md](contracts/configuration.md): startup arguments, shallow discovery, refresh, and resource bounds.
- [contracts/xtp-reader.md](contracts/xtp-reader.md): supported native grammar, routes, local geometry, recovery, and category rules.
- [contracts/mcp-tools.md](contracts/mcp-tools.md): additive existing-tool behaviour, four new tools, queries, paging, and errors.
- [quickstart.md](quickstart.md): future executable validation and traceability to every requirement and success criterion.

## Project Structure

### Documentation produced now

```text
specs/002-track-parameter-catalog/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── checklists/requirements.md
└── contracts/
    ├── configuration.md
    ├── xtp-reader.md
    └── mcp-tools.md
```

`tasks.md` is intentionally deferred to `$speckit-tasks`, which follows this completed planning phase.

### Proposed implementation touch points

```text
src/
├── main.rs                 # parse explicit startup configuration, serve STDIO
├── lib.rs                  # expose owned catalogue and native adapter
├── layout.rs               # retain placed objects and existing unit types
├── xtc.rs                  # preserve reader; no catalogue parsing here
├── catalog.rs              # definitions, snapshots, resolution, selection, queries
├── xtp.rs                  # native parameter reader and trustworthy recovery
└── mcp.rs                  # typed tools and coherent session replacement
tests/
├── catalog_tests.rs        # identity, resolution, filters, pages, refresh, limits
├── xtp_tests.rs            # native grammar, routes, recovery, exact geometry
├── config_tests.rs         # source arguments and portable discovery behaviour
├── mcp_stdio_tests.rs      # extend actual executable workflows
└── fixtures/catalog/       # independently authored XTP/XTC fixtures and manifest
```

**Structure Decision**: Two focused new modules. `xtp` hides native record syntax; `catalog` hides discovery/index/resolution/query mechanics behind owned operations. `mcp` owns protocol mapping and a single coherent session. Reuse unit/segment types only where their semantics agree; catalogue endpoints do not have placed-track connection references. Avoid database/repository abstractions and broad parser refactors.

## Implementation sequence for task generation

1. Establish configuration and authored fixture/provenance contracts; add failing native-reader cases.
2. Implement owned definitions and XTP framing, preserving full segment indexes, signed routes, polygons, and source locations. Validate primitives/topology before claiming completeness.
3. Implement bounded source snapshots, deterministic identities, and recovery/coverage reporting. Prove exact part values and unchanged files.
4. Implement conservative resolution, atomic explicit selection, filters, bounded pages, and stale-token rules.
5. Integrate serialized session open/clear/refresh and the four typed tools. Preserve all 001 tests; extend discovery expectations from three to seven tools.
6. Exercise the real executable, source mutation/reopen cases, missing and partial libraries, and limits. Validate the pinned real compatibility source locally, update README and dependency attribution, then record actual results.

**Exit gate**: Every acceptance case in [quickstart.md](quickstart.md) passes, old inspection scenarios pass, the pinned library's 43 entries and three sample identities are checked, and fmt/Clippy/locked tests/release build pass. No implementation or completion claim is made by this document.
