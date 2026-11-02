# Contract: Supported XTrackCAD Parameter Definitions

Status: planned reader subset. Primary evidence and pinned source links are in [research.md](../research.md), R1–R4 and R8. This grammar is independent of the format-12 XTC layout reader.

## File and object framing

- Accept UTF-8 `.xtp`/`.XTP` text, LF or CRLF, blank lines, and `#` comment lines. First noncomment record must be a nonempty `CONTENTS` header. Preserve the first label as canonical and subsequent `CONTENTS` values as aliases. `SUBCONTENTS` labels apply to subsequent objects.
- Optional `PARAM 12` is accepted once, before definitions; absence is valid and reported as null. Malformed/duplicate directives invalidate the library with `malformed_parameter_file`; other explicit integer versions produce `unsupported_parameter_version`. A `VERSION` directive is not an XTP version declaration and is unsupported.
- Track definitions use `TURNOUT <scale> "<manufacturer>\t<description>\t<part-number>" [flags]`. The displayed `\t` denotes actual tab delimiters. Preserve the entire native title. Parse structured fields only for three valid fields with an optional trailing empty tab; otherwise keep native label and expose unestablished fields as null.
- Support native doubled-quote escaping in header titles; do not apply JSON escaping rules to the source. Supported P names contain no embedded quote: the native P scanner differs from the header decoder. Unverified escape variants produce a diagnostic rather than a guessed title or route.
- No flags or numeric zero flags are supported. Preserve nonzero flags such as `512` but mark their semantics unsupported and the definition partial; do not infer slip/adjustable behaviour.
- `END` and `END$SEGS` each terminate a definition, with no trailing noncomment data. `STRUCTURE` and `CARPROTO` are known non-track terminated blocks; `CARPART` is a known non-track single-line record. Count their object ordinals and frame/skip them without returning them as track parts.
- `INCLUDE` is diagnosed as unsupported and never followed. Unknown file directives, including `CHECKSUM`, are not silently trusted: retain the issue and mark inventory incomplete if their effect or framing cannot be established. No implicit reads outside configured sources.

## Supported track body

| Record | Supported interpretation |
|---|---|
| `E x y angle` | Local endpoint position and outward direction, finite values |
| `S color width x1 y1 x2 y2` | Local straight centerline with distinct endpoints |
| `C color width radius cx cy start sweep` | Local circular centerline, nonzero signed radius and sweep; absolute sweep at most 360 degrees |
| `P "name" signed-index...` | Named route state with ordered traversals; see below |
| `U ...` | Recognized informational designer data; retained as source bytes, no effect on geometry/category |

Require at least two supported endpoints and one track segment for complete geometry. Numeric fields must be finite, indexes/counts valid, and computed geometry within finite range. No invented elevation, gauge, length, or derived manufacturer dimensions. Gauge remains null for this subset.

`X adjustable`, joint/Bezier/nested track records, unknown primitives, or unsupported endpoint forms preserve recoverable identity but make the relevant geometry/routes incomplete. Metadata remains accessible even when detail arrays must be null.

## Segment slots and decorative framing

Maintain one native index for each drawable segment, including artwork; endpoints, route states, and informational records do not consume segment indexes. Track detail returns supported S/C slots with their native indexes, never a renumbered S/C-only sequence.

Recognize legacy `L/A/G/F/Y`, versioned `L3/A3/G3`, and `F3/F4/Y3/Y4` decorative records. Parse their documented numeric field shapes sufficiently to frame and validate them, then exclude them from exported track geometry. An F/Y polygon consumes one slot plus its declared continuation rows: legacy vertices use XY, F3/Y3 vertices use XY plus an elevation placeholder (require zero in this planar subset), and F4/Y4 vertices use XY plus integer vertex type; version-four headers include polygon type. The vertex count is bounded and must match; continuation rows are not new segment slots. Do not extrapolate one version's field count onto another.

Unknown artwork can be omitted only if its framing and single-slot contribution are proven. Otherwise geometry/routes become incomplete, since later path references cannot safely be resolved. Intentionally excluded, correctly framed decoration does not itself invalidate complete track centerlines.

## Signed paths and validation

1. References are signed, one-based indexes into the full native drawable array. Positive `n` enters slot `n` at its native endpoint zero; negative `-n` enters endpoint one. Reject out-of-range indexes and references to non-track slots.
2. Zero separates simultaneous traversals within one named path state. Return nested traversal arrays; never connect across zero. Reject empty traversals, leading/trailing zero, and consecutive separators for the supported subset. The native reader supplies its own internal terminators; they are not textual route references.
3. For S, native endpoints are its declared first and second coordinates. For positive-radius C, endpoint zero is at start angle and endpoint one at start plus sweep; negative radius reverses this endpoint ordering. Use radius magnitude for the locus while retaining its sign.
4. Validate each traversal's adjacent positional joins and its outer endpoints against declared E positions using `1e-5` inch for six-decimal rounding. Require an unambiguous declared endpoint at each outer end for complete routes. Preserve endpoint directions; this validation does not promise external connection compatibility.
5. Missing or malformed P records produce incomplete routes, not inferred routes. Preserve separately known complete geometry. Names are preserved; route identity/order is source-based, so duplicate names do not merge states.

## Conservative category rules

Assign a non-null category only when geometry and routes are complete and all track primitives are accounted for in the interpreted route topology:

- `straight`: two endpoints and one S track primitive connecting them.
- `curve`: two endpoints and one C track primitive connecting them.
- `turnout`: three endpoints with two alternative endpoint-to-endpoint traversals sharing exactly one external endpoint and reaching distinct other endpoints. A shared S/C primitive is not required: the representative 24612 uses separate S and C primitives meeting at one endpoint. More complicated topology remains unclassified.
- `crossing`: four endpoints with two independent through traversals joining disjoint endpoint pairs and no interpreted switching connection between those paths. A named state may contain concurrent zero-separated traversals.
- Everything else: `category: null` with available native metadata. Do not label unknown or flagged compound parts as switches solely because their native type is `TURNOUT`.

These are conservative query categories, not a general topology solver. Category uncertainty alone need not invalidate otherwise completely understood geometry/routes.

The two-endpoint requirement intentionally leaves single-ended buffer stops partial in this first subset. The `1e-5` inch continuity tolerance is Turnout policy for reliable returned routes, not XTrackCAD's more permissive native connection acceptance threshold.

## Recovery and completeness

Maintain source span, definition bytes, ordinal, diagnostic code, and line location. A malformed/unsupported definition with a known header and proven termination can remain indexed with partial/unsupported status. Resume at the proven terminator or next recognized column-one top-level object/header. If a new header arrives before the old terminator, diagnose the truncated prior record before resuming.

Do not resynchronize on arbitrary keywords inside quotes, vertex payloads, or unknown nested blocks. If framing becomes uncertain, stop that region, mark `inventory_complete: false`, and retain only previously trustworthy entries. Library-file failure must remain a catalogue diagnostic, never an XTC layout-open failure.

Definition status and geometry/route completeness are independent of whole-source enumeration. Whole-file unknown content cannot be hidden merely because known parts were recovered. All detail responses must make these distinctions explicit.
