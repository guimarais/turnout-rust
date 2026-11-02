---

description: "Executable task list for Track Parameter Catalogue Retrieval"
---

# Tasks: Track Parameter Catalogue Retrieval

**Input**: Design documents from `/specs/002-track-parameter-catalog/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [quickstart.md](quickstart.md), and [contracts/](contracts/)

**Tests**: Tests are required by FR-017 and the implementation plan. Write each listed test before the associated production implementation, verify it fails for the intended missing behavior, then make it pass.

**Organization**: Tasks are grouped by user story so each increment is independently testable. Existing feature 001 behavior remains a compatibility constraint throughout.

## Path Conventions

- Rust sources: `src/`
- Rust integration tests: `tests/`
- Independently authored catalogue fixtures and provenance: `tests/fixtures/catalog/`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish the locked dependencies, explicit startup configuration boundary, and authored fixture corpus required by all stories.

- [X] T001 Add locked direct `sha2` 0.11 and `getrandom` 0.4 dependencies, resolve `Cargo.lock`, and record their required attribution in `Cargo.toml`, `Cargo.lock`, and `THIRD_PARTY_LICENSES.md`
- [ ] T002 Create the independently authored fixture manifest with GPL-3.0-only provenance, SHA-256 values, expected record counts, exact geometry/routes, and expected diagnostics in `tests/fixtures/catalog/README.md`
- [X] T003 [P] Create unique four-part and one-part layout fixtures, including exact straight, curve, turnout, and crossing expectations, in `tests/fixtures/catalog/system-a.xtp` and `tests/fixtures/catalog/one-part.xtc`
- [ ] T004 [P] Create colliding, duplicate, mixed-scale, and empty/custom-only catalogue fixtures in `tests/fixtures/catalog/system-b.xtp`, `tests/fixtures/catalog/duplicate-source.xtp`, `tests/fixtures/catalog/mixed.xtc`, and `tests/fixtures/catalog/empty.xtc`
- [ ] T005 [P] Create authored legacy/versioned grammar, malformed/partial, UTF-8, source-limit, and response-limit fixtures in `tests/fixtures/catalog/`
- [ ] T006 Add `--library-source PATH` OS-argument parsing with startup-working-directory capture and the 64-source startup limit tests in `tests/config_tests.rs`
- [X] T007 Implement startup configuration parsing that accepts only repeated `--library-source PATH` arguments and rejects unknown, missing/empty, and over-limit arguments without protocol stdout in `src/main.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the owned, bounded catalogue domain and native XTP reader that every user-story operation relies upon.

**⚠️ CRITICAL**: Complete this phase before exposing or integrating any catalogue MCP tool.

- [X] T008 Define owned source, library, part, local-geometry, endpoint, route, diagnostic, completeness, selection, query, and cursor types with explicit nullable JSON fields in `src/catalog.rs`
- [ ] T009 Implement versioned length-prefixed SHA-256 library/part/snapshot identities, process nonce/session counter, source fingerprints, and opaque cursor encoding/validation primitives in `src/catalog.rs`
- [ ] T010 [P] Add native-reader tests for UTF-8 `CONTENTS`/aliases/groups, unversioned and `PARAM 12` framing, quoted titles, structured fields, terminators, and source locations in `tests/xtp_tests.rs`
- [ ] T011 [P] Add native-reader tests for S/C endpoints, full drawable native indexes, decorative legacy/versioned polygons, signed route references, zero-separated traversals, and conservative category topology in `tests/xtp_tests.rs`
- [ ] T012 [P] Add native-reader tests for flags, unsupported primitives, invalid routes, malformed/truncated records, unsafe resynchronization, byte/record/slot/route limits, and completeness diagnostics in `tests/xtp_tests.rs`
- [ ] T013 Implement bounded UTF-8 XTP file/object framing for `CONTENTS`, optional `PARAM 12`, `SUBCONTENTS`, `TURNOUT`, recognized non-track records, `END`/`END$SEGS`, and source spans in `src/xtp.rs`
- [ ] T014 Implement native header/title decoding, valid tab-separated metadata extraction, flags, finite numeric validation, endpoint and S/C local geometry parsing, and explicit unavailable gauge in `src/xtp.rs`
- [ ] T015 Implement full drawable-slot accounting and proven framing for L/A/G/F/Y, L3/A3/G3, F3/F4/Y3/Y4 decoration while exporting only supported S/C track segments in `src/xtp.rs`
- [ ] T016 Implement signed one-based route parsing, zero-separated traversal validation, 1e-5-inch joins/outer-endpoint checks, and no-inference completeness rules in `src/xtp.rs`
- [ ] T017 Implement trustworthy per-definition recovery and source-level incomplete-inventory handling for unsupported/malformed/unknown-framing content in `src/xtp.rs`
- [ ] T018 Implement conservative straight, curve, turnout, crossing, and null category classification from complete supported geometry and route topology in `src/xtp.rs`
- [X] T019 Add the native XTP adapter export and owned catalogue module exports without changing XTC format-12 parsing semantics in `src/lib.rs`, `src/xtp.rs`, and `src/catalog.rs`

**Checkpoint**: The application can represent and parse a bounded native parameter snapshot without modifying layout semantics or source files.

---

## Phase 3: User Story 1 — Retrieve available parts (Priority: P1) 🎯 MVP

**Goal**: After opening a layout, automatically expose an unambiguous matching library and its complete catalogue, including unplaced definitions and exact local detail.

**Independent Test**: Open `one-part.xtc` with `system-a.xtp`; verify automatic evidence/selection and retrieve all four definitions, including the three unplaced entries, with exact local geometry, endpoints, routes, units, and completeness.

### Tests for User Story 1

- [ ] T020 [P] [US1] Add catalogue tests for bounded explicit file/directory discovery, ASCII-case-insensitive `.xtp`, canonical deduplication, deterministic order, bounded reads, snapshot fingerprints, and unchanged source bytes/mtimes in `tests/catalog_tests.rs`
- [ ] T021 [P] [US1] Add catalogue tests for identity-and-scale tuple evidence, unambiguous automatic selection, unplaced definitions, mixed systems, duplicate records, and no inferred gauge/compatibility in `tests/catalog_tests.rs`
- [ ] T022 [P] [US1] Extend executable STDIO tests for the additive `catalogue` field on successful `open_layout`/`get_layout_info`, four-part discovery, exact detail data, and preserved three-tool feature-001 behavior in `tests/mcp_stdio_tests.rs`

### Implementation for User Story 1

- [ ] T023 [US1] Implement shallow configured-source discovery, canonical-path deduplication, sorted bounded immutable byte snapshots, changed-during-read detection, and source availability/coverage diagnostics in `src/catalog.rs`
- [ ] T024 [US1] Build library and part snapshots from `src/xtp.rs`, retaining source-specific identities, fingerprints, evidence-ready structured identities, source locations, support counts, and definition/source completeness in `src/catalog.rs`
- [ ] T025 [US1] Implement exact placed `(scale, manufacturer, description, part_number)` matching, bounded evidence, and automatic selection only for complete unambiguous inventory in `src/catalog.rs`
- [X] T026 [US1] Replace the layout-only state with one serialized layout/catalogue session that clears before open, snapshots only after valid XTC parsing, and publishes a fresh coherent session in `src/mcp.rs`
- [ ] T027 [US1] Add shared catalogue-state and library/part-detail rendering with explicit units, nullable unavailable values, source-specific provenance, and response-budget-safe compact catalogue status in `src/mcp.rs`
- [X] T028 [US1] Implement `list_track_libraries` and `get_track_part` typed MCP tools with current-session checks, selected-library enforcement for detail, and equivalent structured JSON/text results in `src/mcp.rs`

**Checkpoint**: A uniquely matching library can be inspected through STDIO without altering placed geometry or either source file.

---

## Phase 4: User Story 2 — Select libraries and search parts (Priority: P2)

**Goal**: Allow explicit multi-library selection and deterministic search/paging while retaining distinct duplicate source definitions.

**Independent Test**: Open `empty.xtc`, select two libraries with colliding part numbers, filter each supported field, then page at limit one and receive every expected source-specific entry exactly once.

### Tests for User Story 2

- [ ] T029 [P] [US2] Add catalogue tests for ambiguity/candidate evidence, explicit multi-selection from an empty layout, atomic invalid replacement, explicit clear, and selection-revision lifecycle in `tests/catalog_tests.rs`
- [ ] T030 [P] [US2] Add catalogue tests for AND filters, Unicode-lowercase matching, category/scale semantics, duplicate identity retention, deterministic order, limits, and no-match/zero-entry pages in `tests/catalog_tests.rs`
- [ ] T031 [P] [US2] Add catalogue tests for valid, malformed, stale, changed-query, changed-limit, changed-selection, and out-of-range pagination cursors in `tests/catalog_tests.rs`
- [ ] T032 [P] [US2] Extend executable STDIO selection/search workflow coverage for colliding libraries, all filters, stable page traversal, invalid selection preservation, and explicit clear in `tests/mcp_stdio_tests.rs`

### Implementation for User Story 2

- [X] T033 [US2] Implement atomic sorted/deduplicated current-snapshot library replacement, unavailable/unknown validation, explicit-clear state, origin, and selection-revision updates in `src/catalog.rs`
- [ ] T034 [US2] Implement canonical part query normalization, selected-subset validation, Unicode lowercase filters, exact category/scale rules, deterministic library-ID/ordinal ordering, and indexed totals in `src/catalog.rs`
- [ ] T035 [US2] Implement opaque query-bound library and part pagination cursors with page-size binding, offset-after-last-entry behavior, invalid/stale diagnostics, and bounded response reduction in `src/catalog.rs`
- [ ] T036 [US2] Implement `select_track_libraries` and `list_track_parts` typed MCP tools with contract argument schemas, expected domain failures, and bounded result rendering in `src/mcp.rs`

**Checkpoint**: Ambiguous or custom layouts can select and browse multiple libraries predictably, without silent selection or duplicate merging.

---

## Phase 5: User Story 3 — Inspect safely despite catalogue limitations (Priority: P3)

**Goal**: Keep valid layout inspection and recoverable catalogue metadata available when sources fail, change, or contain unsupported content.

**Independent Test**: Open valid layouts against missing, malformed, partial, changed, and resource-limited sources; verify diagnostics/completeness, immutable active snapshots, session isolation, and unchanged source files.

### Tests for User Story 3

- [ ] T037 [P] [US3] Add configuration tests for missing/unreadable paths, shallow directory failure, unrelated/nested entries, child symlink skipping, duplicate aliases, path spaces/Unicode, and every configured discovery/read cap in `tests/config_tests.rs`
- [ ] T038 [P] [US3] Add catalogue tests for partial/unsupported recoverable entries, unsafe-boundary inventory incompleteness, diagnostic/evidence truncation, response-limit behavior, source mutation/deletion snapshots, and reopen refresh in `tests/catalog_tests.rs`
- [ ] T039 [P] [US3] Extend executable STDIO lifecycle tests for no configuration, valid open with catalogue failures, failed-open clearing, new-open isolation, stale tokens, source integrity, and the unchanged feature-001 workflow in `tests/mcp_stdio_tests.rs`

### Implementation for User Story 3

- [ ] T040 [US3] Complete source-status handling for unavailable directories/files, skipped symlinks, invalid UTF-8, aggregate/per-file/record limits, bounded diagnostics, and explicit global inventory coverage in `src/catalog.rs`
- [ ] T041 [US3] Enforce immutable-session browsing and lifecycle invalidation on reopen, failed open, server restart, and successful selection while preserving failed-selection state in `src/catalog.rs` and `src/mcp.rs`
- [ ] T042 [US3] Apply new-tool diagnostic/evidence/JSON response limits so oversized metadata or detail arrays return explicit `response_limit_exceeded` with false response completeness rather than silently truncated complete data in `src/mcp.rs`
- [ ] T043 [US3] Preserve feature-001 open/info/list result fields, format-12 parser behavior, layout fingerprint, and no-open failure lifecycle while adding only catalogue-specific results in `src/mcp.rs`, `src/layout.rs`, and `src/xtc.rs`

**Checkpoint**: Catalogue limitations never corrupt or block valid read-only layout inspection, and no stale session data can be used.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Complete documentation, provenance, real-source validation, and release-quality verification.

- [ ] T044 [P] Document the implemented startup flags, four catalogue tools, matching/filter/paging rules, source limits, snapshot lifecycle, supported XTP subset, and unavailability semantics in `README.md` and `docs/track-parameter-catalog.md`
- [ ] T045 [P] Update implementation dependency and fixture provenance/redistribution records, retaining the external pinned library as non-vendored, in `THIRD_PARTY_LICENSES.md` and `tests/fixtures/catalog/README.md`
- [ ] T046 Validate the separately supplied pinned `mrklnhoc-de.xtp` checksum, 43-entry enumeration, sample identities, complete supported categories, flagged double-slip limitation, and unchanged source/layout bytes; record environment and results in `specs/002-track-parameter-catalog/verification.md`
- [ ] T047 Run formatting, Clippy warnings-as-errors, all locked tests including feature-001 regression coverage, locked release build, and record actual Rust/Cargo versions/results in `specs/002-track-parameter-catalog/verification.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately; T001 and T007 establish build/startup prerequisites, while fixture tasks can proceed in parallel.
- **Foundational (Phase 2)**: Begins after T001 and fixture scaffolding; T013–T18 depend on their corresponding failing tests and T008’s owned types. It blocks user-story integration.
- **US1 (Phase 3)**: Depends on the foundational reader/domain. It supplies snapshot/session behavior that US2 and US3 extend.
- **US2 (Phase 4)**: Depends on US1 snapshot/session and library rendering; selection/query mechanics are independently testable against its fixtures.
- **US3 (Phase 5)**: Depends on US1 session/snapshot behavior and US2 cursor/selection mechanics for full lifecycle coverage.
- **Polish (Phase 6)**: Depends on the desired user stories being complete.

### User Story Dependencies

```text
Setup → Foundational → US1 (MVP) → US2 → US3 → Polish
```

- **US1** delivers the first useful catalogue increment and is the recommended MVP.
- **US2** builds on current-library snapshots to alter selection and query them; it must not weaken US1 automatic resolution.
- **US3** hardens the same snapshot/session model across failure and change cases and verifies feature-001 compatibility.

### Parallel Opportunities

- T003–T006 can run in parallel after the dependency decision in T001.
- T010–T012 are independent test files/areas and can run in parallel before reader implementation.
- Within US1, T020–T022 can run in parallel; within US2, T029–T032 can run in parallel; within US3, T037–T039 can run in parallel.
- T044 and T045 can be completed in parallel after behavior is implemented.

## Parallel Example: User Story 2

```text
Task: "Add selection atomicity and explicit-clear tests in tests/catalog_tests.rs"
Task: "Add filter/page/cursor tests in tests/catalog_tests.rs using a separate module section"
Task: "Extend actual STDIO selection/search workflow tests in tests/mcp_stdio_tests.rs"
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete setup and the bounded native-reader/domain foundation.
2. Complete US1 through `get_track_part` and verify the four-part unique-match STDIO workflow.
3. Stop and validate that sources and placed layout geometry remain unchanged.

### Incremental Delivery

1. Add US1 automatic discovery, immutable snapshots, and detail retrieval.
2. Add US2 explicit selection, search, paging, and cursor semantics.
3. Add US3 failure/change hardening and compatibility regression coverage.
4. Finish documentation, pinned-source evidence, and release validation.

## Notes

- Every task uses the required checkbox, sequential task ID, story label where applicable, and exact file path(s).
- `[P]` marks work that can proceed in parallel without relying on incomplete work in the same execution step.
- The contracts remain authoritative for exact bounds and all nullable/completeness semantics.
