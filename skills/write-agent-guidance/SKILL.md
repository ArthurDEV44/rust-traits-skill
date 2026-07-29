---
name: write-agent-guidance
description: Create, audit, consolidate, and migrate repository instruction chains centered on a canonical AGENTS.md with minimal CLAUDE.md import bridges. Use for AGENTS.md, AGENTS.override.md, CLAUDE.md, nested agent guidance, Codex or Claude instruction-scope conflicts, or deciding whether durable guidance belongs in an instruction file, linked documentation, a skill, or mechanical enforcement.
---

# Write Agent Guidance

Build one evidence-backed instruction chain. Treat `AGENTS.md` as the canonical
cross-agent source and `CLAUDE.md` as a thin Claude Code adapter. Optimize for
instruction adherence, not documentation completeness.

Read [the authoring rubric](references/authoring-rubric.md) before evaluating or
changing guidance. It contains the current provider semantics, placement rules,
bridge patterns, and acceptance checks.

## 1. Fix the scope

Classify the request as:

- **Author or migrate**: inspect, write the requested files, and validate them.
- **Audit or explain**: inspect and report; leave files unchanged.

Resolve the target as global, repository root, or nested subtree. Infer it from
the requested paths and repository boundary. Ask only when choosing the wrong
scope would place durable instructions outside the user's intended authority.

Discover the effective instruction chain with targeted file searches. Include
applicable ancestors and inspect:

- `AGENTS.md` and `AGENTS.override.md`
- `CLAUDE.md` and `CLAUDE.local.md`
- `.claude/rules/**/*.md`
- `.codex/config.toml` when fallback names or byte limits matter

Keep global configuration out of scope for a repository request, and keep
repository files out of scope for a global request.

**Complete when:** the target directory, repository boundary, existing
instruction files, and load order for both Codex and Claude Code are explicit.

## 2. Build an evidence ledger

Read only the repository artifacts needed to verify candidate instructions:
manifests, task scripts, CI workflows, architecture documents, generated-file
markers, and the nearest source files. Prefer repository evidence over prose
that may have drifted.

For every candidate rule, record:

1. the behavior it changes;
2. its evidence or explicit user source;
3. its narrowest valid scope;
4. how an agent can verify compliance;
5. the destination selected by the rubric.

Exclude facts an agent can cheaply infer, generic quality advice, temporary task
details, duplicated tool enforcement, secrets, and speculative conventions.

**Complete when:** every retained rule has a source, scope, behavioral effect,
and verification signal.

## 3. Author the canonical chain

Write `AGENTS.md` as a map, not an encyclopedia. Use direct, imperative,
repository-specific language. State exact commands and boundaries. Pair hard
guardrails with the safe path or exception. Link to detailed documentation
instead of copying it.

Order content by consequence:

1. critical safety and authorization boundaries;
2. exact setup, build, test, and validation commands;
3. architecture and ownership boundaries that code does not reveal;
4. scoped workflow and delivery expectations;
5. pointers to detailed knowledge.

Create nested `AGENTS.md` files only when a rule genuinely applies to a
subtree. Rewrite contradictions as explicit scopes rather than relying on
last-instruction-wins behavior.

Keep each Claude-loaded file below Anthropic's 200-line target and the combined
Codex project chain below the configured byte limit, 32 KiB by default.

**Complete when:** every line earns always-loaded context and each meaning has
one authoritative location.

## 4. Bridge Claude Code

For colocated repository files, make `CLAUDE.md` start with:

```md
@AGENTS.md
```

Append a `## Claude Code` section only for irreducible Claude-specific behavior.
At global scope, calculate the real relative import from `CLAUDE.md` to the
canonical `AGENTS.md`; do not assume the files are adjacent. Pair each nested
`AGENTS.md` with a nested bridge when Claude must receive that subtree guidance.

Treat `AGENTS.override.md` as a parity hazard: Codex may select it instead of
`AGENTS.md`, while a static Claude import does not switch automatically. Prefer
stable `AGENTS.md` scope for shared guidance. If an override is required,
surface the divergence and make the intended temporary behavior explicit.

**Complete when:** Claude resolves the intended canonical file, the import graph
has no cycle, and provider-specific additions are not duplicated.

## 5. Compress and verify

Reconstruct the effective chain for the root and every representative nested
working directory. Remove duplicated, stale, generic, conflicting, or
mechanically enforced prose. Confirm that every referenced path and command
exists and that no secret or private local detail entered shared guidance.

Run the skill validator when this skill itself is being changed. For generated
guidance, perform static validation first. Run fresh Codex or Claude sessions
only when the user authorized the associated external execution or cost:

- Codex: ask it to list loaded instruction files in order and summarize the
  effective rules from the root and a representative nested directory.
- Claude Code: inspect `/context` for the expected memory files, then verify the
  effective rules from the same directories.

Inspect the final files or diff. Report the canonical source, scopes created,
material rules excluded, and validation evidence. For an audit, report proposed
changes without applying them.

**Complete when:** both effective chains are coherent, within budget, grounded
in repository evidence, and validated to the maximum authorized level.
