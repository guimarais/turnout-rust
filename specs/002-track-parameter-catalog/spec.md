# Feature Specification: Track Parameter Catalogue Retrieval

**Feature identifier**: `002-track-parameter-catalog`

**Created**: 2026-10-31

**Status**: Specified and planned; not implemented.

**Input**: Plan feature 002 from the project documentation: after loading a layout, retrieve the straights, curves, turnouts, crossings, and other parts registered in its track-system parameter libraries. Write Spec-Kit artifacts only.

## User Scenarios & Testing

### User Story 1 — Retrieve available parts (Priority: P1)

A designer opens a layout and asks which catalogue parts are available, including parts that have not yet been placed.

**Why this priority**: Complete layout design requires actual reusable definitions beyond the instances already placed.

**Independent Test**: Open a layout using one part from a library containing a straight, curve, turnout, and crossing; retrieve all four definitions.

**Acceptance Scenarios**:

1. **Given** configured libraries and an unambiguous supported identity-and-scale match, **When** a layout opens, **Then** matching libraries are selected automatically with evidence, without claiming proof of the original source.
2. **Given** a selected four-part library and one placed part, **When** its catalogue is listed, **Then** all four definitions are accessible, including the three unplaced parts.
3. **Given** a supported definition, **When** details are requested, **Then** the result contains native identity, provenance, scale, local track geometry, endpoints, named routes when defined, units, and completeness information.
4. **Given** mixed systems or scales, **When** libraries are selected, **Then** their parts remain distinguishable and scale/gauge compatibility is not inferred.

### User Story 2 — Select libraries and search parts (Priority: P2)

A designer chooses libraries explicitly and finds parts by manufacturer, catalogue number, description, scale, or category.

**Why this priority**: Custom layouts, mixed systems, and ambiguous identities cannot rely solely on automatic resolution.

**Independent Test**: Open an empty layout, select two libraries with colliding numbers, and filter/page through their distinct definitions.

**Acceptance Scenarios**:

1. **Given** two libraries matching the same placed identity and scale, **When** resolution runs, **Then** both candidates and their evidence are visible and neither is silently chosen.
2. **Given** an empty/custom-only layout, **When** libraries are explicitly selected, **Then** unplaced parts become accessible without requiring a placed match.
3. **Given** duplicate part numbers within or across libraries, **When** parts are listed, **Then** every definition retains a distinct source-specific identity.
4. **Given** selected libraries, **When** each supported filter and successive pages are requested, **Then** matching follows documented rules and stable bounded pages contain exactly the expected entries without gaps or duplicates.
5. **Given** a valid selection, **When** a replacement contains an unknown/unavailable library, **Then** the entire request fails with a diagnostic and the previous selection remains intact.
6. **Given** any selection, **When** it is explicitly cleared, **Then** automatic resolution does not override that choice before another layout open.

### User Story 3 — Inspect safely despite catalogue limitations (Priority: P3)

A designer can inspect the existing layout when external libraries are absent, damaged, or partly unsupported.

**Why this priority**: Embedded layout definitions remain independently useful; catalogue uncertainty must not corrupt them.

**Independent Test**: Open layouts with missing, malformed, and partly supported library sources and verify diagnostics, recoverable entries, session isolation, and unchanged source files.

**Acceptance Scenarios**:

1. **Given** no configured sources or missing/unreadable libraries, **When** a valid layout opens, **Then** inspection succeeds and catalogue status explains the limitation.
2. **Given** supported and unsupported/malformed definitions with trustworthy boundaries, **When** a library is browsed, **Then** recoverable supported entries remain available and affected entries expose source locations and completeness.
3. **Given** unsafe record boundaries or a resource limit, **When** indexing stops, **Then** coverage is explicitly incomplete and no partial definition appears complete.
4. **Given** an open session, **When** another layout opens or an open fails, **Then** prior selections, evidence, and continuation tokens cannot be used as current results.
5. **Given** a library changes during browsing, **When** details are requested, **Then** they retain the opened snapshot; reopening refreshes the source identity and resolution.
6. **Given** successful, failed, or partial operations, **When** sources are checked, **Then** layout and library bytes and modification times are unchanged.
7. **Given** an existing feature 001 client without configured libraries, **When** its inspection workflow runs, **Then** existing arguments, result fields, compatibility rules, and failure lifecycle remain supported.

### Edge Cases

- Uppercase `.XTP`, spaces/non-ASCII paths, duplicate sources, aliases, and duplicate definitions.
- Configured directories with unrelated files, nested directories, symlinks, or unreadable entries.
- Missing, unstructured, or changed layout labels; multiple `CONTENTS` labels and optional groups.
- Native `TURNOUT` records describing straights or crossings rather than physical switches.
- Unsupported adjustable/nested geometry, malformed routes, decoration references, or multiline polygons.
- Missing gauge, non-finite numbers, invalid encoding, truncated blocks, or unrecognized version directives.
- Stale tokens after selection changes, changed queries, reopening, or server restart.
- No matches, no selected libraries, empty libraries, or selected libraries with only unsupported entries.

## Requirements

### Functional Requirements

- **FR-001**: The system MUST accept explicitly configured local parameter files/directories and restrict discovery to those sources, without requiring an XTrackCAD process or network retrieval.
- **FR-002**: Successful layout opening MUST produce library candidates using available placed-part identity and scale, with matching evidence and uncertainty. Automatic selection MUST require unambiguous matches in a completely enumerated source inventory; names alone MUST NOT prove original provenance.
- **FR-003**: Users MUST be able to select multiple available libraries or clear selection, including for empty/custom-only layouts and libraries without placed matches. Invalid replacement MUST preserve the previous selection atomically.
- **FR-004**: The system MUST distinguish unconfigured, no-match, candidates, explicitly unselected, and selected states, reporting selection origin and source/definition completeness separately.
- **FR-005**: Selected catalogues MUST expose all discoverable track-part entries, including unplaced and flagged unsupported entries, without merging duplicate names or numbers.
- **FR-006**: The system MUST support manufacturer, number, description, scale, and category filters with documented matching, bounded deterministic pages, and rejection of invalid/stale continuation tokens.
- **FR-007**: Libraries and definitions MUST have source-specific identities, paths/locations, labels, and content fingerprints for the exact snapshot. Unchanged sources MUST retain identities across opens.
- **FR-008**: Supported definitions MUST preserve valid UTF-8 labels/native types and expose available identity, scale, explicit gauge, local track geometry, endpoints/directions, and named routes with explicit units/conventions. Unavailable values MUST be explicit.
- **FR-009**: Physical category MUST be distinct from native record type and assigned only from supported geometry/topology. Unestablished categories MUST remain unavailable. Scale MUST NOT imply gauge or connection compatibility.
- **FR-010**: Malformed/unsupported content MUST produce source-located diagnostics and completeness information. Safely recoverable entries MUST remain accessible; incomplete geometry/routes MUST NOT be presented as complete.
- **FR-011**: Missing, unreadable, malformed, or ambiguous libraries MUST NOT fail valid layout inspection or replace embedded placed geometry.
- **FR-012**: Each layout open MUST refresh catalogue sources and resolution, discarding previous choices. Failed opens MUST leave no usable layout-specific catalogue state; startup configuration MAY persist.
- **FR-013**: Browsing MUST use one consistent snapshot until the next layout open; changed files and fingerprints MUST become visible on reopening.
- **FR-014**: All operations MUST preserve layout and parameter-file bytes and modification times.
- **FR-015**: Discovery, parsing, diagnostics, and result sizes MUST have documented bounds; exceeded limits MUST be diagnosed and incomplete coverage explicit.
- **FR-016**: Existing feature 001 inputs/result fields, strict layout format 12, independent application-version reporting, explicit unavailable values, source integrity, and failed-open clearing MUST remain compatible. Catalogue additions MUST be separately identifiable.
- **FR-017**: Automated validation MUST cover supported part categories, unplaced/mixed/ambiguous/duplicate parts, malformed/unsupported data, selection/pagination lifecycle, refresh, limits, integrity, and the real client workflow on Linux. Retained fixtures MUST record origin and redistribution terms.

### Key Entities

- **Library source**: Explicitly configured file or directory.
- **Library snapshot**: Source identity, labels, discoverable entries, availability, and diagnostics captured at layout open.
- **Part definition**: Reusable source-specific identity, metadata, local geometry/endpoints/routes, and completeness; distinct from a placed instance.
- **Resolution evidence**: Placed identity associated with candidate libraries, ambiguity, and unmatched information.
- **Selection**: Current libraries and automatic/explicit selection origin.
- **Catalogue query**: Filters and continuation state tied to the current snapshot and selection.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A layout using one part allows retrieval of all four straight/curve/turnout/crossing fixture definitions, including three unplaced parts, with exact expected geometry and units.
- **SC-002**: Mixed/duplicate fixtures retain every expected source definition; ambiguous fixtures produce zero silent source choices.
- **SC-003**: Every missing/unreadable-library scenario preserves valid layout inspection, and malformed/unsupported fixtures expose expected source locations and completeness.
- **SC-004**: Paging each tested filter returns exactly its expected entries once; all invalid-selection and stale-token scenarios follow the defined lifecycle.
- **SC-005**: Every tested operation preserves source bytes and modification times; existing feature 001 regression scenarios continue to pass.
- **SC-006**: An assistant completes open → select libraries → search parts → retrieve definition through the supported client connection without conversion, downloads, or XTrackCAD running.

## Assumptions

- Feature 001 is complete. [Catalogue requirements](../../docs/track-parameter-catalog.md) and [project context](../../TURNOUT_PROJECT_CONTEXT.md) establish this follow-on scope; 002 does not reopen 001.
- Local `.xtp` libraries are supplied by the operator. Explicit directory discovery is new 002 scope, independent of 001's historical no-scanning restriction.
- Embedded definitions remain authoritative for placed geometry. Candidate catalogue matches are not proof of original source or permission to replace geometry.
- Supported syntax is a bounded fixture-proven subset. Full historical parameter-format support is not promised.
- Configuration is established at startup. Selection belongs to an open layout; reopening refreshes sources.
- Linux is the first validated platform; paths/file handling must remain portable.
- Placement, connection solving, editing/writing layouts or libraries, exports/previews, downloads, and importing XTrackCAD configuration are excluded.
