# Research: Track Parameter Catalogue Retrieval

Date: 2026-10-31. Decisions below are design choices unless explicitly identified as native-format facts. Research used separate agents for native format, Rust/MCP integration, and documentation coverage; no implementation files were changed.

## Documentation reviewed and authority

| Documentation | Effect on 002 |
|---|---|
| `TURNOUT_PROJECT_CONTEXT.md`, especially 5.1, 6.7, 8, milestone 2a | Requires catalogue retrieval before catalogue-based creation, multiple systems/scales, explicit units, portability, and immutable sources |
| `docs/track-parameter-catalog.md` | Primary feature brief: twelve required behaviours, eight acceptance cases, four proposed tools, and bounded exclusions |
| `README.md` | Implemented baseline is three read-only STDIO tools; external libraries are not implemented |
| 001 `spec.md`, `plan.md`, `research.md` | Completed inspection scope, owned reader/model, strict layout format 12, dependency licensing boundary |
| 001 `data-model.md`, `contracts/read-only-tools.md` | Existing response fields, explicit nulls/units, diagnostics, source order, and session clearing are compatibility constraints |
| 001 `tasks.md`, `quickstart.md`, checklist | All 43 tasks complete; 42 tests and release build recorded; catalogue work is explicitly separate |
| `THIRD_PARTY_LICENSES.md` | Project GPL-3.0-only, locked dependency provenance; do not import the GPL-2.0 parser without a separate decision |
| `layouts/manifest.json`, authored fixture README | Supplied sample's provenance is recorded but external redistribution unverified; authored fixtures have explicit project terms |
| `.specify/memory/constitution.md` | Unfilled template, not a ratified source of additional gates |
| Spec-Kit templates/scripts, init options, local feature state | Sequential 002 directory; resolved templates; feature identity independent of Git branch; planning ends before tasks |

The context still contains one historical “TrackBrain” milestone sentence, tentative technology questions, and a suggested module tree. These do not override its authoritative Turnout name or 001's implemented architecture. Historical 40-test notes in 001 precede its final 42-test closeout. Leave completed 001 and general project documents unchanged in this documentation-only request.

## R1 — Separate catalogue definitions from placed instances

**Decision**: Add an owned catalogue model and native XTP adapter; retain the existing layout model and embedded world geometry. Matching establishes candidates, never original provenance or permission to replace instances.

**Rationale**: XTrackCAD copies reusable definitions into layouts. Track systems can span files and layouts can mix them. Native `TURNOUT` covers sectional track, not only switches. A library has labels/aliases and optional groups. These are native facts; our separate model and conservative matching are design inferences.

**Alternatives considered**: Treat layout labels as paths; infer an entire catalogue from placed parts; require XTrackCAD's runtime/configuration. All fail for absent, mixed, custom, or renamed libraries.

**Source**: [Official parameter-file information](https://sourceforge.net/p/xtrkcad-fork/wiki/ParameterFileInformation/), Basic Information, File Headers, Object Headers. The operational rules below are Turnout decisions rather than claims from that reference.

## R2 — Use the native parameter grammar, not the layout grammar

**Decision**: Support unversioned legacy files and explicit `PARAM 12`, with first noncomment header `CONTENTS`, aliases, groups, `TURNOUT` definitions, supported local primitives/endpoints/routes, and both native terminators. Reject other explicit parameter versions as unsupported. Do not require or interpret an XTC `VERSION` header.

**Rationale**: The inspected parameter reader initializes its parameter version to an absent value and recognizes `PARAM`, while the current source version constant is 13. Choosing unversioned plus 12 is a conservative initial Turnout subset, not a claim that upstream rejects other versions. This covers the first real library without conflating XTC/XTP versions.

**Alternatives considered**: Require `VERSION 12` (wrong grammar); reject unversioned files (excludes real target); promise all parameter versions accepted upstream (untested).

**Sources**: Pinned [parameter reader](https://github.com/adbyrne/XTrkCAD/blob/6058992cb3a97d97ba373dfcfb7f17ccc52aef15/app/bin/paramfile.c#L239), [version constants](https://github.com/adbyrne/XTrkCAD/blob/6058992cb3a97d97ba373dfcfb7f17ccc52aef15/xtrkcad-config.h.in#L40). These are an inspected XTrackCAD source snapshot, not a promise of compatibility with every subsequent revision.

## R3 — Preserve native route indexing and direction

**Decision**: Retain the full drawable-segment sequence, including decorative slots. Resolve signed, one-based path references against it, and represent each named path state as one or more traversals separated by zero. Validate range, track-kind targets, and positional continuity; preserve native signs and indexes in results.

**Rationale**: A filtered S/C-only array changes indexes. A positive reference enters native endpoint zero, a negative reference endpoint one. Zero separates concurrent traversals, not an instruction to connect them. Curve radius sign affects native endpoint ordering while its magnitude determines the locus. The model must represent all of these independently.

**Alternatives considered**: Flatten all route numbers into one connected chain; drop signs; index track segments only; infer routes from labels. Each can invent connectivity.

**Sources**: [Signed segment decoding](https://github.com/adbyrne/XTrkCAD/blob/6058992cb3a97d97ba373dfcfb7f17ccc52aef15/app/bin/compound.c#L632), [segment/path parser](https://github.com/adbyrne/XTrkCAD/blob/6058992cb3a97d97ba373dfcfb7f17ccc52aef15/app/bin/trkseg.c#L1212), [native segment endpoints](https://github.com/adbyrne/XTrkCAD/blob/6058992cb3a97d97ba373dfcfb7f17ccc52aef15/app/bin/trkseg.c#L87).

## R4 — Recover conservatively and classify from topology

**Decision**: Parse supported decorative framing to preserve segment indexes, including counted legacy/versioned polygons. Isolate malformed definitions only at a proven terminator or recognized column-one top-level header. Unknown framing taints enumeration. Do not follow `INCLUDE`. Use complete track geometry/routes for conservative categories; otherwise return null.

**Rationale**: Geometry rows are not standalone objects. Artwork may be omitted from track detail without invalidating known centerlines, but unknown segment framing or references cannot be silently skipped. Native type alone does not establish a physical category.

**Alternatives considered**: Silently discard unknown content; reject a whole source for every unsupported part; port the full upstream object system. The chosen subset retains useful metadata without overstating support.

**Source**: [Native segment parser and polygon framing](https://github.com/adbyrne/XTrkCAD/blob/6058992cb3a97d97ba373dfcfb7f17ccc52aef15/app/bin/trkseg.c#L1401). Recovery, completeness, and classification rules are Turnout policy specified in [the reader contract](contracts/xtp-reader.md).

## R5 — Explicit configuration and refresh on open

**Decision**: Repeat `--library-source PATH`; accept a file or shallow directory, deduplicate canonical file paths, and snapshot on each successful layout open. Resolve relative paths against captured startup working directory. Browsing does no source I/O. Reopening refreshes; failed opens clear all session data.

**Rationale**: Matches the existing one-session server and user expectation “once the layout file is loaded.” A snapshot keeps fingerprints and details consistent. Limits and read failures are catalogue diagnostics, not layout failures.

**Alternatives considered**: Environment-variable path lists (platform delimiter ambiguity); XTrackCAD config import (stale mappings and extra discovery scope); startup-only snapshots (requires restart for changes); watchers/background caches/refresh tools (unnecessary lifecycle complexity).

## R6 — Stable source identities and bounded queries

**Decision**: SHA-256 for new catalogue content/identity fields; stable library ID from canonical source identity, part ID from library ID plus source ordinal and record fingerprint. Process-random nonce plus open counter identifies sessions. Bind cursors to session, selection revision, exact normalized filter, and offset. Keep the existing layout FNV fingerprint unchanged.

**Rationale**: Part numbers are not unique, array position alone is not durable identity, and default hashers do not define a stable public algorithm. Tokens must become stale on new opens even when bytes are unchanged. Caps, pagination, and diagnostic truncation prevent unbounded replies.

**Alternatives considered**: Manufacturer/number as key; content-only library IDs merging distinct sources; migration to one global identity scheme; signed authorization tokens. None is necessary for a local read-only catalogue query.

**Sources**: Maintainer documentation for [sha2](https://docs.rs/sha2/0.11.1/sha2/) and [getrandom](https://docs.rs/getrandom/0.4.3/getrandom/) establishes SHA-256 and portable OS randomness availability and declared compatible license choices. Versions are planning selections; lockfile resolution and attribution happen during implementation.

## R7 — Preserve the MCP and dependency baseline

**Decision**: Retain RMCP 0.8.5. Add `catalogue` to successful open/info results and four typed tools. Keep layout diagnostics separate from catalogue diagnostics; preserve the existing three-tool inputs and `list_tracks` output. Use a coherent session mutex, with no await while its guard is held.

**Rationale**: Local SDK inspection confirms structured/structured-error helpers already provide structured content and equivalent JSON text. Typed parameter deserialization already supplies invalid-argument errors. A dependency upgrade is not required.

**Alternatives considered**: Replace the SDK, add HTTP, manually emit protocol messages, or redesign all tools. These add unrelated risk.

**Evidence**: Repository `Cargo.lock` and `src/mcp.rs`; cached RMCP 0.8.5 `src/model.rs` (`CallToolResult`) and `src/handler/server/tool.rs` (`Parameters`). These are integration findings from the locked implementation, not claims about newer SDK versions.

## R8 — First real compatibility target and fixture provenance

**Decision**: Reference, but do not copy into this planning change, [`mrklnhoc-de.xtp`](https://github.com/adbyrne/XTrkCAD/blob/6058992cb3a97d97ba373dfcfb7f17ccc52aef15/app/lib/params/mrklnhoc-de.xtp) at revision `6058992cb3a97d97ba373dfcfb7f17ccc52aef15`. Use independently authored catalogue fixtures for routine tests; perform a separately recorded local compatibility run against this pinned source before declaring 002 complete.

**Rationale**: Inspected bytes contain 43 track definitions and exact matches for sample part identities 24229, 24130, and 24612. They exercise legacy framing, multiple curve/turnout parts, crossings, concurrent routes, and a flagged double slip. File SHA-256: `d76b90fd573e7b7d33b9fd806fb338f8097e081b268fd9ad2d729b3c6b6611c5`. This is research evidence, not a passing Turnout parser result. Some definitions will deliberately remain partial under the bounded subset.

**Alternatives considered**: A different English-labelled Märklin file (does not establish exact sample-label coverage); copying arbitrary upstream fixtures (unverified redistribution terms); only synthetic fixtures (misses native serializer variants).

No upstream library is redistributed or runtime-downloaded by this feature. Before any future vendoring, record exact origin, revision, checksum, applicable data terms, and required notices. Keeping the externally supplied file outside the repository avoids making vendoring a prerequisite. The GPL-2.0 parser remains excluded as a production dependency.

## Resolution summary

Configuration, scope, parser subset, matching policy, categories, identity, refresh, limits, and error contracts are decided in the accompanying artifacts. No unanswered product clarification blocks task generation. The real-source compatibility run, dependency attribution, and all executable validation are future implementation gates, not unresolved design choices or completed tests.
